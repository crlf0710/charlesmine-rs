#[cfg(not(windows))]
fn compile_resource() {
    // do nothing
}

#[cfg(windows)]
#[path = "src/view_assets_catalog.rs"]
pub(crate) mod resource_catalog;

#[cfg(windows)]
fn compile_resource() {
    use resource_catalog as catalog;
    use resw::*;

    Build::with_two_languages(lang::LANG_CHS)
        .resource(
            catalog::IDI_CHARLESMINE,
            resource::Icon::from_file("./res/CharlesMine.ico"),
        )
        .resource(
            catalog::IDC_CHARLESMINE,
            resource::Accelerators::from_builder()
                .event(
                    catalog::IDM_HELP_ABOUT,
                    accelerators::Event::ascii_key_event(
                        accelerators::ASCIIKey::ascii_key(b'/'),
                        accelerators::ASCIIModifier::Alt,
                    ),
                )
                .event(
                    catalog::IDM_HELP_ABOUT,
                    accelerators::Event::ascii_key_event(
                        accelerators::ASCIIKey::ascii_key(b'?'),
                        accelerators::ASCIIModifier::Alt,
                    ),
                )
                .event(
                    catalog::IDM_FILE_NEW,
                    accelerators::Event::virt_key_event(
                        accelerators::VirtKey::F2,
                        accelerators::Modifier::None,
                    ),
                )
                .event(
                    catalog::IDM_ADVANCED_RESTART,
                    accelerators::Event::virt_key_event(
                        accelerators::VirtKey::F8,
                        accelerators::Modifier::None,
                    ),
                )
                .event(
                    catalog::IDM_ADVANCED_RECORD_STOP,
                    accelerators::Event::virt_key_event(
                        accelerators::VirtKey::F12,
                        accelerators::Modifier::None,
                    ),
                )
                .event(
                    catalog::IDM_ADVANCED_LOADMAP,
                    accelerators::Event::virt_key_event(
                        accelerators::VirtKey::F5,
                        accelerators::Modifier::None,
                    ),
                )
                .event(
                    catalog::IDM_ADVANCED_SAVEMAP,
                    accelerators::Event::virt_key_event(
                        accelerators::VirtKey::F6,
                        accelerators::Modifier::None,
                    ),
                )
                .build(),
        )
        .resource(
            catalog::IDB_BLOCKS,
            resource::Bitmap::from_file("./res/Blocks.bmp"),
        )
        .resource(
            catalog::IDB_BUTTON,
            resource::Bitmap::from_file("./res/Button.bmp"),
        )
        .resource(
            catalog::IDB_DIGIT,
            resource::Bitmap::from_file("./res/Digit.bmp"),
        )
        .resource(
            catalog::IDC_CHARLESMINE,
            resource::Menu::from_builder()
                .popup(
                    MultiLangText::from("&Game").lang(lang::LANG_CHS, "游戏(&G)"),
                    |popup| {
                        popup
                            .item(
                                catalog::IDM_FILE_NEW,
                                MultiLangText::from("&New\tF2")
                                    .lang(lang::LANG_CHS, "开局(&N)\tF2"),
                            )
                            .separator()
                            .item(
                                catalog::IDM_FILE_GAME_EASY,
                                MultiLangText::from("&Beginner").lang(lang::LANG_CHS, "初级(&B)"),
                            )
                            .item(
                                catalog::IDM_FILE_GAME_MEDIUM,
                                MultiLangText::from("&Intermediate")
                                    .lang(lang::LANG_CHS, "中级(&I)"),
                            )
                            .item(
                                catalog::IDM_FILE_GAME_HARD,
                                MultiLangText::from("&Expert").lang(lang::LANG_CHS, "高级(&E)"),
                            )
                            .item(
                                catalog::IDM_FILE_GAME_CUSTOM,
                                MultiLangText::from("&Custom...")
                                    .lang(lang::LANG_CHS, "自定义(&C)..."),
                            )
                            .separator()
                            .item(
                                catalog::IDM_FILE_MARK,
                                MultiLangText::from("&Marks (?)")
                                    .lang(lang::LANG_CHS, "标记(?)(&M)"),
                            )
                            .separator()
                            .item(
                                catalog::IDM_FILE_HERO_LIST,
                                MultiLangText::from("Best &Times...")
                                    .lang(lang::LANG_CHS, "扫雷英雄榜(&T)..."),
                            )
                            .separator()
                            .item(
                                catalog::IDM_FILE_EXIT,
                                MultiLangText::from("E&xit").lang(lang::LANG_CHS, "退出(&X)"),
                            )
                    },
                )
                .popup(
                    MultiLangText::from("&Advanced").lang(lang::LANG_CHS, "高级(&A)"),
                    |popup| {
                        popup
                            .item(
                                catalog::IDM_ADVANCED_LOADMAP,
                                MultiLangText::from("&Load Game\tF5")
                                    .lang(lang::LANG_CHS, "加载雷局(&L)\tF5"),
                            )
                            .item(
                                catalog::IDM_ADVANCED_SAVEMAP,
                                MultiLangText::from("&Save Game\tF6")
                                    .lang(lang::LANG_CHS, "保存雷局(&S)\tF6"),
                            )
                            .separator()
                            .item(
                                catalog::IDM_ADVANCED_RESTART,
                                MultiLangText::from("&Restart Game\tF8")
                                    .lang(lang::LANG_CHS, "重新开始本局(&R)\tF8"),
                            )
                            .separator()
                            .item(
                                catalog::IDM_ADVANCED_RECORD_RECORD,
                                MultiLangText::from("Start R&ecording")
                                    .lang(lang::LANG_CHS, "开始录像(&E)"),
                            )
                            .item(
                                catalog::IDM_ADVANCED_RECORD_PLAY,
                                MultiLangText::from("Start &Playback")
                                    .lang(lang::LANG_CHS, "开始回放(&P)"),
                            )
                            .item(
                                catalog::IDM_ADVANCED_RECORD_STOP,
                                MultiLangText::from("S&top Recording/Playback\tF12")
                                    .lang(lang::LANG_CHS, "停止(&T)\tF12"),
                            )
                            .separator()
                            .item(
                                catalog::IDM_ADVANCED_ZOOM_1x,
                                MultiLangText::from("Zoom 1x").lang(lang::LANG_CHS, "缩放 1x"),
                            )
                            .item(
                                catalog::IDM_ADVANCED_ZOOM_2x,
                                MultiLangText::from("Zoom 2x").lang(lang::LANG_CHS, "缩放 2x"),
                            )
                            .item(
                                catalog::IDM_ADVANCED_ZOOM_3x,
                                MultiLangText::from("Zoom 3x").lang(lang::LANG_CHS, "缩放 3x"),
                            )
                    },
                )
                .popup(
                    MultiLangText::from("&Help").lang(lang::LANG_CHS, "帮助(&H)"),
                    |popup| {
                        popup.item(
                            catalog::IDM_HELP_ABOUT,
                            MultiLangText::from("&About CharlesMine...")
                                .lang(lang::LANG_CHS, "关于 钻石扫雷(&A)..."),
                        )
                    },
                )
                .build(),
        )
        .resource(
            catalog::IDD_ABOUTBOX,
            resource::Dialog::from_builder()
                .system_menu()
                .caption(MultiLangText::from("About").lang(lang::LANG_CHS, "关于"))
                .font(
                    "Tahoma",
                    FontSize::pt(9),
                    FontWeight::default(),
                    FontItalic::default(),
                    FontCharset::default(),
                )
                .lang_specific_font(
                    lang::LANG_CHS,
                    "SimSun",
                    FontSize::pt(9),
                    FontWeight::default(),
                    FontItalic::default(),
                    FontCharset::default(),
                )
                .style(dialog::DialogStyle::MODAL_FRAME)
                .rect(Rect::new(22, 17, 186, 60))
                .control(
                    catalog::IDC_MYICON,
                    dialog::Control::from_template(dialog::ControlTemplate::ICON)
                        .image_id(catalog::IDI_CHARLESMINE)
                        .rect(Rect::new(14, 9, 21, 21)),
                )
                .control(
                    catalog::IDC_TEXT1,
                    dialog::Control::from_template(dialog::ControlTemplate::LTEXT)
                        .text(
                            MultiLangText::from("CharlesMine 1.0")
                                .lang(lang::LANG_CHS, "钻石扫雷 1.0"),
                        )
                        .rect(Rect::new(49, 10, 119, 8))
                        .style(dialog::StaticControlStyle::NO_PREFIX),
                )
                .control(
                    catalog::IDC_TEXT2,
                    dialog::Control::from_template(dialog::ControlTemplate::LTEXT)
                        .text(
                            MultiLangText::from("CrLF0710 Home-made")
                                .lang(lang::LANG_CHS, "CrLF0710 制作"),
                        )
                        .rect(Rect::new(49, 20, 119, 8)),
                )
                .control(
                    predefined_id::OK,
                    dialog::Control::from_template(dialog::ControlTemplate::DEFPUSHBUTTON)
                        .text(MultiLangText::from("OK"))
                        .rect(Rect::new(75, 32, 36, 15))
                        .style(dialog::WindowStyle::GROUP),
                )
                .build(),
        )
        .compile()
        .expect("Failed to compile resource");
}

fn main() {
    compile_resource();
}
