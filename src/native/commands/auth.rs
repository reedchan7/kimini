use std::time::Duration;

use gpui::{AppContext, Context, Window};

use crate::protocol::{OAuthFlowStart, OAuthFlowStatus};

use super::super::app::{SettingsTab, Shell, UtilityPanel};

impl Shell {
    pub(in crate::native) fn toggle_auth_panel(&mut self, cx: &mut Context<Self>) {
        self.utility_panel = if self.utility_panel == Some(UtilityPanel::Auth) {
            None
        } else {
            Some(UtilityPanel::Auth)
        };
        if self.utility_panel == Some(UtilityPanel::Auth) {
            self.settings_tab = SettingsTab::General;
            self.refresh_auth(cx);
        }
        cx.notify();
    }

    pub(in crate::native) fn open_auth_panel(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        let opened = self.utility_panel != Some(UtilityPanel::Auth);
        self.utility_panel = Some(UtilityPanel::Auth);
        self.settings_tab = tab;
        if opened {
            self.refresh_auth(cx);
        }
        cx.notify();
    }

    pub(in crate::native) fn refresh_auth(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.auth.loading = true;
        self.auth.error = None;
        let task = cx.background_spawn(async move { client.auth_summary() });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                this.auth.loading = false;
                match result {
                    Ok(summary) => this.auth.summary = Some(summary),
                    Err(error) => this.auth.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn start_oauth_login(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        if self.auth.loading {
            return;
        }
        self.auth.loading = true;
        self.auth.error = None;
        let task = cx.background_spawn(async move { client.start_oauth_login() });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                this.auth.loading = false;
                match result {
                    Ok(flow @ OAuthFlowStart::Pending { .. }) => {
                        this.auth.begin_flow(flow);
                        this.schedule_auth_poll(cx);
                    }
                    Ok(OAuthFlowStart::Authenticated { .. }) => {
                        this.auth.clear_flow();
                        this.refresh_auth(cx);
                    }
                    Err(error) => this.auth.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn open_oauth_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((url, _, _)) = self.auth.pending() else {
            return;
        };
        let url = url.to_owned();
        self.open_browser_url(&url, window, cx);
    }

    pub(in crate::native) fn cancel_oauth_login(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.auth.loading = true;
        self.auth.error = None;
        let task = cx.background_spawn(async move { client.cancel_oauth_login() });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                this.auth.loading = false;
                match result {
                    Ok(_) => {
                        this.auth.clear_flow();
                        this.refresh_auth(cx);
                    }
                    Err(error) => this.auth.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn logout_oauth(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.auth.loading = true;
        self.auth.error = None;
        let task = cx.background_spawn(async move { client.logout_oauth() });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                this.auth.loading = false;
                match result {
                    Ok(_) => {
                        this.auth.clear_flow();
                        this.refresh_auth(cx);
                    }
                    Err(error) => this.auth.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn schedule_auth_poll(&mut self, cx: &mut Context<Self>) {
        let Some((_, _, interval)) = self.auth.pending() else {
            return;
        };
        if self.auth.poll_scheduled {
            return;
        }
        self.auth.poll_scheduled = true;
        let generation = self.auth.poll_generation;
        let timer = cx
            .background_executor()
            .timer(Duration::from_secs(interval.clamp(1, 30)));
        cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |this, cx| {
                if generation != this.auth.poll_generation {
                    return;
                }
                this.auth.poll_scheduled = false;
                this.poll_oauth_login(cx);
            });
        })
        .detach();
    }

    fn poll_oauth_login(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let generation = self.auth.poll_generation;
        let task = cx.background_spawn(async move { client.oauth_login_status() });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if generation != this.auth.poll_generation {
                    return;
                }
                match result {
                    Ok(Some(snapshot)) if snapshot.status == OAuthFlowStatus::Pending => {
                        this.schedule_auth_poll(cx);
                    }
                    Ok(Some(snapshot)) if snapshot.status == OAuthFlowStatus::Authenticated => {
                        this.auth.clear_flow();
                        this.refresh_auth(cx);
                    }
                    Ok(Some(snapshot)) => {
                        let fallback = match snapshot.status {
                            OAuthFlowStatus::Denied => this.strings.native.auth_denied,
                            OAuthFlowStatus::Expired => this.strings.native.auth_expired,
                            OAuthFlowStatus::Cancelled => this.strings.native.auth_cancelled,
                            OAuthFlowStatus::Pending | OAuthFlowStatus::Authenticated => {
                                this.strings.native.auth_failed
                            }
                        };
                        this.auth.clear_flow();
                        this.auth.error =
                            Some(snapshot.error_message.unwrap_or_else(|| fallback.into()));
                        cx.notify();
                    }
                    Ok(None) => {
                        this.auth.clear_flow();
                        this.auth.error = Some(this.strings.native.auth_cancelled.into());
                        cx.notify();
                    }
                    Err(error) => {
                        this.auth.error = Some(error);
                        this.schedule_auth_poll(cx);
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }
}
