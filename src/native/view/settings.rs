use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px, relative, rgba};
use gpui_component::{Icon, IconName, Sizable as _, StyledExt, scroll::ScrollableElement};

use super::super::app::{AccentMode, AppearanceMode, DefaultPermission, SettingsTab, Shell};
use super::super::commands::config::ConfigPreference;
use super::super::theme::*;
use super::panel::panel_button;
use super::settings_components::*;

impl Shell {
    pub(super) fn auth_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-overlay")
            .role(Role::Group)
            .aria_label(self.strings.settings_title)
            .absolute()
            .inset_0()
            .p_4()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000055))
            .child(
                div()
                    .w(px(760.0))
                    .max_w(relative(0.94))
                    .h(px(672.0))
                    .max_h(relative(0.92))
                    .flex()
                    .flex_col()
                    .rounded_xl()
                    .border_1()
                    .border_color(theme_rgb(BORDER_STRONG))
                    .bg(theme_rgb(SURFACE))
                    .shadow_xl()
                    .child(self.settings_header(cx))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .child(self.settings_tabs(cx))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .min_h_0()
                                    .border_l_1()
                                    .border_color(theme_rgb(BORDER))
                                    .overflow_y_scrollbar()
                                    .px_5()
                                    .pt(px(26.0))
                                    .pb_5()
                                    .child(self.settings_content(cx)),
                            ),
                    ),
            )
    }

    fn settings_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(54.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .border_b_1()
            .border_color(theme_rgb(BORDER))
            .child(
                div()
                    .text_size(font_px(13.0))
                    .font_semibold()
                    .child(self.strings.settings_title),
            )
            .child(
                div()
                    .id("close-settings")
                    .focusable()
                    .tab_stop(true)
                    .role(Role::Button)
                    .aria_label(self.strings.native.close_auth)
                    .cursor_pointer()
                    .size(px(30.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .border_1()
                    .border_color(theme_rgb(BORDER))
                    .text_color(theme_rgb(TEXT_MUTED))
                    .hover(|item| {
                        item.bg(theme_rgb(SURFACE_ACTIVE))
                            .text_color(theme_rgb(TEXT))
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.utility_panel = None;
                        cx.notify();
                    }))
                    .child(Icon::new(IconName::Close).xsmall()),
            )
    }

    fn settings_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(148.0))
            .flex_none()
            .p_2()
            .flex()
            .flex_col()
            .gap_1()
            .child(self.settings_tab_button(
                SettingsTab::General,
                self.strings.native.settings_general,
                cx,
            ))
            .child(self.settings_tab_button(
                SettingsTab::Agent,
                self.strings.native.settings_agent,
                cx,
            ))
            .child(self.settings_tab_button(
                SettingsTab::Account,
                self.strings.native.settings_account,
                cx,
            ))
            .child(self.settings_tab_button(
                SettingsTab::Advanced,
                self.strings.native.settings_advanced,
                cx,
            ))
            .child(self.settings_tab_button(
                SettingsTab::Archived,
                self.strings.native.archived_sessions,
                cx,
            ))
    }

    fn settings_tab_button(
        &self,
        tab: SettingsTab,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.settings_tab == tab;
        div()
            .id(("settings-tab", tab as usize))
            .focusable()
            .tab_stop(true)
            .role(Role::Tab)
            .aria_label(label)
            .aria_selected(selected)
            .cursor_pointer()
            .rounded_md()
            .px_3()
            .py_2()
            .text_size(font_px(13.0))
            .text_color(theme_rgb(if selected { ACCENT } else { TEXT_SECONDARY }))
            .when(selected, |item| item.bg(theme_rgb(ACCENT_SOFT)))
            .hover(|item| {
                item.bg(theme_rgb(SURFACE_ACTIVE))
                    .text_color(theme_rgb(TEXT))
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.settings_tab = tab;
                if tab == SettingsTab::Archived {
                    this.ensure_archived_sessions_loaded(cx);
                }
                cx.notify();
            }))
            .child(label)
    }

    fn settings_content(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.settings_tab {
            SettingsTab::General => self.settings_general(cx).into_any_element(),
            SettingsTab::Agent => self.settings_agent(cx).into_any_element(),
            SettingsTab::Account => self.settings_account(cx).into_any_element(),
            SettingsTab::Advanced => self.settings_advanced(cx).into_any_element(),
            SettingsTab::Archived => self.settings_archived(cx).into_any_element(),
        }
    }

    fn settings_general(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = self.strings.native;
        let appearance = div()
            .w(px(270.0))
            .flex()
            .items_center()
            .rounded_md()
            .overflow_hidden()
            .child(
                settings_segment(
                    "appearance-light",
                    strings.settings_moon_bright,
                    self.preferences.appearance == AppearanceMode::MoonBright,
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.update_theme_preferences(
                        |preferences| preferences.appearance = AppearanceMode::MoonBright,
                        window,
                        cx,
                    );
                })),
            )
            .child(
                settings_segment(
                    "appearance-dark",
                    strings.settings_moon_dark,
                    self.preferences.appearance == AppearanceMode::MoonDark,
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.update_theme_preferences(
                        |preferences| preferences.appearance = AppearanceMode::MoonDark,
                        window,
                        cx,
                    );
                })),
            )
            .child(
                settings_segment(
                    "appearance-system",
                    strings.settings_system,
                    self.preferences.appearance == AppearanceMode::System,
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.update_theme_preferences(
                        |preferences| preferences.appearance = AppearanceMode::System,
                        window,
                        cx,
                    );
                })),
            );
        let accent = div()
            .w(px(116.0))
            .flex()
            .items_center()
            .rounded_md()
            .overflow_hidden()
            .child(
                settings_segment(
                    "accent-blue",
                    strings.settings_blue,
                    self.preferences.accent == AccentMode::Blue,
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.update_theme_preferences(
                        |preferences| preferences.accent = AccentMode::Blue,
                        window,
                        cx,
                    );
                })),
            )
            .child(
                settings_segment(
                    "accent-black",
                    strings.settings_black,
                    self.preferences.accent == AccentMode::Black,
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.update_theme_preferences(
                        |preferences| preferences.accent = AccentMode::Black,
                        window,
                        cx,
                    );
                })),
            );
        let font_size = div()
            .h(px(30.0))
            .flex()
            .items_center()
            .rounded_md()
            .border_1()
            .border_color(theme_rgb(BORDER_STRONG))
            .overflow_hidden()
            .child(
                settings_stepper_button("font-smaller", "−").on_click(cx.listener(
                    |this, _, window, cx| {
                        this.update_theme_preferences(
                            |preferences| {
                                preferences.font_size =
                                    preferences.font_size.saturating_sub(1).max(12);
                            },
                            window,
                            cx,
                        );
                    },
                )),
            )
            .child(
                div()
                    .w(px(48.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(font_px(12.0))
                    .child(format!("{} px", self.preferences.font_size)),
            )
            .child(
                settings_stepper_button("font-larger", "+").on_click(cx.listener(
                    |this, _, window, cx| {
                        this.update_theme_preferences(
                            |preferences| {
                                preferences.font_size =
                                    preferences.font_size.saturating_add(1).min(20);
                            },
                            window,
                            cx,
                        );
                    },
                )),
            );
        let language = div()
            .w(px(126.0))
            .flex()
            .items_center()
            .rounded_md()
            .overflow_hidden()
            .child(
                settings_segment("language-en", "English", self.lang == crate::i18n::Lang::En)
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.lang != crate::i18n::Lang::En {
                            this.toggle_language(window, cx);
                        }
                    })),
            )
            .child(
                settings_segment("language-zh", "中文", self.lang == crate::i18n::Lang::Zh)
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.lang != crate::i18n::Lang::Zh {
                            this.toggle_language(window, cx);
                        }
                    })),
            );

        div().flex().flex_col().child(
            settings_section(strings.settings_appearance)
                .child(settings_action_row(
                    strings.settings_color_scheme,
                    appearance,
                ))
                .child(settings_action_row(strings.settings_accent, accent))
                .child(settings_action_row(strings.settings_font_size, font_size))
                .child(settings_action_row(strings.settings_language, language))
                .child(settings_labeled_action_row(
                    strings.settings_show_outline,
                    Some(strings.settings_show_outline_desc),
                    settings_toggle(
                        "conversation-outline",
                        strings.settings_show_outline,
                        self.preferences.conversation_outline,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.update_preferences(
                            |preferences| {
                                preferences.conversation_outline =
                                    !preferences.conversation_outline;
                            },
                            cx,
                        );
                    })),
                )),
        )
    }

    fn settings_agent(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = self.strings.native;
        let Some(config) = self.daemon_config.as_ref() else {
            let detail = self
                .config_error
                .as_deref()
                .map(|error| format!("{}: {error}", strings.settings_config_unavailable))
                .unwrap_or_else(|| strings.settings_config_unavailable.into());
            return settings_section(strings.settings_agent_defaults)
                .child(settings_config_error(detail));
        };
        let current_model = config.default_model.as_deref();
        let model_label = current_model
            .and_then(|model| self.models.iter().find(|item| item.model == model))
            .map(|item| item.label())
            .or(current_model)
            .unwrap_or("-");
        let next_model = next_model_id(&self.models, current_model);
        let saving = self.config_saving;
        let model = settings_select("default-model", model_label)
            .when(saving || next_model.is_none(), disabled_settings_control)
            .when_some(
                (!saving).then_some(next_model).flatten(),
                |control, model| {
                    control.on_click(cx.listener(move |this, _, _, cx| {
                        this.update_config_preference(
                            ConfigPreference::DefaultModel(model.clone()),
                            cx,
                        );
                    }))
                },
            );
        let default_permission = config
            .default_permission_mode
            .as_deref()
            .and_then(DefaultPermission::from_mode);
        let permission = div()
            .w(px(192.0))
            .flex()
            .items_center()
            .rounded_md()
            .overflow_hidden()
            .child(
                settings_segment(
                    "default-permission-manual",
                    strings.permission_manual,
                    default_permission == Some(DefaultPermission::Manual),
                )
                .when(saving, disabled_settings_control)
                .when(!saving, |control| {
                    control.on_click(cx.listener(|this, _, _, cx| {
                        this.update_config_preference(
                            ConfigPreference::DefaultPermission(DefaultPermission::Manual),
                            cx,
                        );
                    }))
                }),
            )
            .child(
                settings_segment(
                    "default-permission-auto",
                    strings.permission_auto,
                    default_permission == Some(DefaultPermission::Auto),
                )
                .when(saving, disabled_settings_control)
                .when(!saving, |control| {
                    control.on_click(cx.listener(|this, _, _, cx| {
                        this.update_config_preference(
                            ConfigPreference::DefaultPermission(DefaultPermission::Auto),
                            cx,
                        );
                    }))
                }),
            )
            .child(
                settings_segment(
                    "default-permission-yolo",
                    strings.permission_yolo,
                    default_permission == Some(DefaultPermission::Yolo),
                )
                .when(saving, disabled_settings_control)
                .when(!saving, |control| {
                    control.on_click(cx.listener(|this, _, _, cx| {
                        this.update_config_preference(
                            ConfigPreference::DefaultPermission(DefaultPermission::Yolo),
                            cx,
                        );
                    }))
                }),
            );
        let default_thinking = config
            .thinking
            .as_ref()
            .and_then(|thinking| thinking.enabled)
            != Some(false);
        let default_plan_mode = config.default_plan_mode == Some(true);
        let merge_skills = config.merge_all_available_skills == Some(true);

        settings_section(strings.settings_agent_defaults)
            .when_some(self.config_error.clone(), |section, error| {
                section.child(settings_config_error(error))
            })
            .child(settings_agent_action_row(
                strings.settings_default_model,
                Some(strings.settings_default_model_desc),
                model,
            ))
            .child(settings_agent_action_row(
                strings.settings_default_permission,
                Some(strings.settings_default_permission_desc),
                permission,
            ))
            .child(settings_agent_action_row(
                strings.settings_default_thinking,
                Some(strings.settings_default_thinking_desc),
                settings_toggle(
                    "default-thinking",
                    strings.settings_default_thinking,
                    default_thinking,
                )
                .when(saving, disabled_settings_control)
                .when(!saving, |control| {
                    control.on_click(cx.listener(move |this, _, _, cx| {
                        this.update_config_preference(
                            ConfigPreference::DefaultThinking(!default_thinking),
                            cx,
                        );
                    }))
                }),
            ))
            .child(settings_agent_action_row(
                strings.settings_default_plan,
                Some(strings.settings_default_plan_desc),
                settings_toggle(
                    "default-plan",
                    strings.settings_default_plan,
                    default_plan_mode,
                )
                .when(saving, disabled_settings_control)
                .when(!saving, |control| {
                    control.on_click(cx.listener(move |this, _, _, cx| {
                        this.update_config_preference(
                            ConfigPreference::DefaultPlanMode(!default_plan_mode),
                            cx,
                        );
                    }))
                }),
            ))
            .child(settings_agent_action_row(
                strings.settings_merge_skills,
                Some(strings.settings_merge_skills_desc),
                settings_toggle("merge-skills", strings.settings_merge_skills, merge_skills)
                    .when(saving, disabled_settings_control)
                    .when(!saving, |control| {
                        control.on_click(cx.listener(move |this, _, _, cx| {
                            this.update_config_preference(
                                ConfigPreference::MergeSkills(!merge_skills),
                                cx,
                            );
                        }))
                    }),
            ))
    }

    fn settings_account(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let summary = self.auth.summary.clone();
        let ready = summary.as_ref().is_some_and(|summary| summary.ready);
        let can_logout = summary
            .as_ref()
            .and_then(|summary| summary.managed_provider.as_ref())
            .is_some();
        let pending_code = self.auth.pending().map(|(_, code, _)| code.to_owned());
        let has_pending = pending_code.is_some();
        settings_section(self.strings.native.settings_account)
            .when_some(
                (!ready).then(|| self.auth.error.clone()).flatten(),
                |panel, error| {
                    panel.child(
                        div()
                            .id("auth-error")
                            .mb_3()
                            .role(Role::Status)
                            .rounded_md()
                            .border_1()
                            .border_color(theme_rgb(ERROR))
                            .p_2()
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(ERROR))
                            .child(error),
                    )
                },
            )
            .when_some(summary, |panel, summary| {
                if summary.ready {
                    let provider = summary
                        .managed_provider
                        .as_ref()
                        .map(|provider| {
                            if provider.name.starts_with("managed:") {
                                provider.name.clone()
                            } else {
                                format!("managed:{}", provider.name)
                            }
                        })
                        .unwrap_or_else(|| "managed".into());
                    let model = summary.default_model.unwrap_or_else(|| "-".into());
                    panel
                        .child(
                            div()
                                .py_2()
                                .text_size(font_px(13.0))
                                .font_medium()
                                .child(provider),
                        )
                        .child(
                            div()
                                .pb_3()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(TEXT_SECONDARY))
                                .child(model),
                        )
                } else {
                    panel.child(self.auth_summary_card(summary))
                }
            })
            .when_some(pending_code, |panel, code| {
                panel.child(self.oauth_handoff(code, cx))
            })
            .when(!has_pending && can_logout, |panel| {
                panel.child(
                    div()
                        .mt_2()
                        .flex()
                        .gap_2()
                        .child(
                            panel_button(
                                self.strings.native.settings_preferences_onboarding,
                                "open-web-preferences",
                            )
                            .border_1()
                            .border_color(theme_rgb(BORDER))
                            .on_click(cx.listener(|this, _, _, cx| this.open_web_fallback(cx))),
                        )
                        .child(
                            panel_button(self.strings.native.sign_out, "oauth-logout")
                                .border_1()
                                .border_color(theme_rgb(BORDER))
                                .on_click(cx.listener(|this, _, _, cx| this.logout_oauth(cx))),
                        ),
                )
            })
            .when(!has_pending && !self.auth.ready(), |panel| {
                panel.child(self.sign_in_action(cx))
            })
    }

    fn settings_advanced(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let endpoint = self
            .connection
            .as_ref()
            .map(|connection| {
                connection
                    .origin()
                    .trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .to_owned()
            })
            .unwrap_or_else(|| "-".into());
        let backend = self
            .server_meta
            .as_ref()
            .map(|meta| match meta.backend.as_str() {
                "v2" => "v2 (kap-server)".to_owned(),
                value => format!("{value} (legacy)"),
            })
            .unwrap_or_else(|| "-".into());
        let server_version = self
            .server_meta
            .as_ref()
            .map(|meta| meta.server_version.clone())
            .unwrap_or_else(|| "-".into());
        let strings = self.strings.native;
        let telemetry = self
            .daemon_config
            .as_ref()
            .map(|config| config.telemetry != Some(false));
        let saving = self.config_saving;
        settings_section(self.strings.native.settings_advanced)
            .child(settings_row(strings.settings_daemon, endpoint))
            .child(settings_row(strings.settings_backend, backend))
            .child(settings_row(
                strings.settings_server_version,
                server_version,
            ))
            .when_some(self.config_error.clone(), |section, error| {
                section.child(settings_config_error(error))
            })
            .when_some(telemetry, |section, telemetry| {
                section.child(settings_labeled_action_row(
                    strings.settings_telemetry,
                    Some(strings.settings_telemetry_desc),
                    settings_toggle("settings-telemetry", strings.settings_telemetry, telemetry)
                        .when(saving, disabled_settings_control)
                        .when(!saving, |control| {
                            control.on_click(cx.listener(move |this, _, _, cx| {
                                this.update_config_preference(
                                    ConfigPreference::Telemetry(!telemetry),
                                    cx,
                                );
                            }))
                        }),
                ))
            })
    }

    fn settings_archived(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let sessions = self.model.archived_sessions().to_vec();
        let empty = sessions.is_empty();
        let has_more = self.model.has_more_archived_sessions();
        settings_section(self.strings.native.archived_sessions)
            .child(
                div()
                    .mb_4()
                    .text_size(font_px(12.0))
                    .text_color(theme_rgb(TEXT_MUTED))
                    .child(self.strings.native.settings_archived_desc),
            )
            .when(empty, |panel| {
                panel.child(
                    div()
                        .py_6()
                        .text_size(font_px(13.0))
                        .text_color(theme_rgb(TEXT_MUTED))
                        .child(if self.archives_loading {
                            self.strings.native.loading_sessions
                        } else {
                            self.strings.native.no_archived_sessions
                        }),
                )
            })
            .children(sessions.into_iter().enumerate().map(|(index, session)| {
                let session_id = session.id.clone();
                let title = if session.title.trim().is_empty() {
                    self.strings.native.untitled_session.to_owned()
                } else {
                    session.title
                };
                div()
                    .py_3()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .border_b_1()
                    .border_color(theme_rgb(BORDER))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .child(div().text_size(font_px(13.0)).line_clamp(1).child(title))
                            .child(
                                div()
                                    .mt_1()
                                    .text_size(font_px(10.0))
                                    .text_color(theme_rgb(TEXT_MUTED))
                                    .line_clamp(1)
                                    .child(session.metadata.cwd),
                            )
                            .child(
                                div()
                                    .mt_0p5()
                                    .text_size(font_px(10.0))
                                    .text_color(theme_rgb(TEXT_MUTED))
                                    .child(session.updated_at),
                            ),
                    )
                    .child(
                        panel_button(
                            self.strings.native.restore_session,
                            ("settings-restore", index),
                        )
                        .border_1()
                        .border_color(theme_rgb(BORDER))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.restore_archived_session(session_id.clone(), cx)
                        })),
                    )
            }))
            .when(has_more, |panel| {
                panel.child(
                    div().mt_3().child(
                        panel_button(
                            self.strings.native.load_more_sessions,
                            "settings-load-more-archives",
                        )
                        .on_click(
                            cx.listener(|this, _, _, cx| this.load_more_archived_sessions(cx)),
                        ),
                    ),
                )
            })
    }
}
