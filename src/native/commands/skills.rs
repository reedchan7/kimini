use gpui::{AppContext, Context};

use crate::api::ApiError;

use super::super::app::{Shell, UtilityPanel};

impl Shell {
    pub(in crate::native) fn toggle_skill_panel(&mut self, cx: &mut Context<Self>) {
        self.utility_panel = if self.utility_panel == Some(UtilityPanel::Skills) {
            None
        } else {
            Some(UtilityPanel::Skills)
        };
        if self.utility_panel == Some(UtilityPanel::Skills) {
            self.refresh_skills(cx);
        }
        cx.notify();
    }

    pub(in crate::native) fn refresh_skills(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.skill_request_generation = self.skill_request_generation.wrapping_add(1);
        let generation = self.skill_request_generation;
        self.skills.begin_load(session_id.clone());
        let request_session = session_id.clone();
        let task = cx.background_spawn(async move { client.list_skills(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if generation != this.skill_request_generation
                    || !this.is_active_session(&request_session)
                {
                    return;
                }
                match result {
                    Ok(list) => {
                        this.skills.install(&request_session, list.skills);
                    }
                    Err(error) => {
                        this.skills.fail(&request_session, error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn activate_skill(
        &mut self,
        name: String,
        args: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.skills.activating = Some(name.clone());
        self.skills.activated = None;
        self.skills.error = None;
        let request_session = session_id.clone();
        let requested_name = name.clone();
        let task = cx.background_spawn(async move {
            client.activate_skill(&session_id, &name, args.as_deref())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error: ApiError| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session) {
                    return;
                }
                this.skills.activating = None;
                match result {
                    Ok(result) if result.activated => {
                        this.skills.activated = Some(result.skill_name)
                    }
                    Ok(_) => {
                        this.skills.error =
                            Some(this.strings.native.skill_activation_unacknowledged.into())
                    }
                    Err(error) => this.skills.error = Some(error),
                }
                if this.skills.activated.as_deref() == Some(&requested_name) {
                    this.reload_active(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }
}
