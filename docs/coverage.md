# Behavior-to-test coverage

| Behavior | Isolated unit coverage |
| --- | --- |
| Individual colors default off; saved true values preserved | `settings::tests::individual_colors_default_off_and_preserve_saved_choice` |
| Fresh special alphabet defaults; saved choices preserved | `settings::tests::fresh_defaults_use_special_and_saved_choice_wins` |
| Compiled speed limits, UI clamping, obsolete settings migration and save cleanup | `settings::tests::speed_hard_limits_are_internal_and_validated`, `terminal::tests::speed_controls_clamp_at_compiled_limits`, `persistence::tests::obsolete_limits_load_but_are_not_saved_or_enforced` |
| Chart sampling cadence, bounded history, empty canvas and occlusion accounting | `metrics::tests::sampling_is_bounded_independent_and_handles_empty_canvas`, `overlapping_traces_count_only_once` |
| Refresh cap displays Hz with rate styling | `terminal::tests::refresh_cap_displays_hertz` |
| Persisted compression defaults, bounds and round-trip | `settings::tests::chart_compression_defaults_persists_and_validates` |
| Logarithmic time scale, inverse labels, preserved live history and control editing | `metrics::tests::compression_preserves_endpoints_order_and_expands_recent_history`, `live_compression_keeps_history_and_density_values`, `terminal::tests::chart_axis_labels_follow_compression_and_available_width` |
| Responsive chart layout and Braille line output | `terminal::tests::chart_layout_and_braille_rendering` |
| Settings serialization, invalid ranges, equal speed bounds | `settings::tests::config_round_trip_and_validation` |
| Rate distribution endpoints, bias and ordering | `settings::tests::bias_preserves_bounds_and_order` |
| Simulation independent of refresh-sized elapsed chunks | `model::tests::cadence_independent` |
| Immediate live palette changes, stable per-glyph rainbow and trace modes, unchanged geometry | `model::tests::live_palettes_and_individual_colors_preserve_scene` |
| Live speed remapping, trace identity | `model::tests::live_speed_changes_preserve_identity_and_percentile` |
| Reserved glyph budget and trace cap | `model::tests::population_respects_reserved_glyph_budget` |
| Empty bounds, resize, zero density and trace expiry | `model::tests::empty_resize_density_and_lifecycle` |
| Depth ordering, non-destructive occlusion and background reveal | `projection::tests::occlusion_is_non_destructive_and_reveals_background` |
| Live controls stay within validated limits | `terminal::tests::controls_validate` |
| 12-row panel, tiny layouts, hidden panel, scrolling selection | `terminal::tests::tiny_layout`, `panel_shows_eight_settings_and_keeps_selection_visible` |
| Green default, live stable highlighting, probability endpoints and projected accents | `model::tests::highlight_probability_is_live_stable_and_projected`, `terminal::tests::green_shades_and_highlight_have_explicit_contrast` |
| Two-shade Blue/Purple/Yellow/Orange palette sampling and RGB contrast | `terminal::tests::individual_profiles_have_two_distinct_shades` |
| Green/Blue labels, two red shades and named profiles | `terminal::tests::named_color_profiles_are_selectable_and_valid` |
| Two-shade Grey profile selection, validation and styled preview | `terminal::tests::grey_profile_and_preview` |
| Complete shared palette and explicit local RGB additions | `terminal::tests::complete_palette_uses_explicit_rgb` |
| Shared dark-theme yellow, distinct from orange | `terminal::tests::yellow_uses_shared_dark_palette` |
| Shared dark-theme orange, distinct from yellow | `terminal::tests::orange_uses_shared_dark_palette` |
| Panel semantic styles, plain-text equivalence, selection, palettes, toggles and save status | `terminal::tests::control_styles_preserve_plain_text_and_selection`, `palette_toggle_and_save_status_have_text_and_color` |
| Failed autosaves retry on exit; successful or unchanged settings do not write again | `terminal::tests::failed_autosave_is_retried_on_exit`, `unchanged_and_successful_saves_do_not_write_again_on_exit` |
| Cleanup resets text/color attributes before cursor/screen restoration | `terminal::tests::terminal_cleanup_resets_style_before_leaving_screen` |
| Rendered output handles one-byte partial writes, broken pipes, write-zero and flush failure | `terminal::tests::rendered_frame_handles_partial_writes_and_output_failures` |
| Narrow foreground over wide background and subsequent reveal | `terminal::tests::narrow_foreground_occludes_wide_background_without_stale_cells` |
| Rendering into Ratatui's isolated test backend | `terminal::tests::frame_contains_glyphs` |
| Default profile directory, legacy fallback, precedence, invalid profile and explicit-path isolation | `persistence::tests::default_profile_location_and_legacy_fallback`, `explicit_missing_file_does_not_inherit_legacy` |
| File save/load and invalid configuration | `persistence::tests::save_load_and_invalid_file` |
| Failed replacement cleans temporary files; invalid saves preserve destination | `persistence::tests::failed_replace_cleans_temporary_and_preserves_destination`, `invalid_save_preserves_existing_settings` |

Gaps: additional exact frame fixtures, randomized simulation time partitions,
and terminal restoration under real terminal errors. These remain work before release. Presets, file
watching and additional appearance profiles are deferred.

Manual checks proposed (not run by the agent): start in a terminal, toggle `?`,
resize including a very small window, change settings, pause/resume, exit via each
supported key, restart to check persistence, and compare both speed mappings.

Milestone validation: 29 isolated unit tests pass; formatting, Clippy and build
pass. Developer visual feedback accepted the green/highlight combination, live
controls, Braille chart and resize behavior. The agent did not run interactive
terminal checks. Remaining gaps above are release follow-up work.

Release preparation also verifies the packaged source against registry term-colors
0.2.0; see [release checks](releasing.md).
