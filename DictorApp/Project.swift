import ProjectDescription

// MARK: - Shared FFI Settings
let sharedFFISettings: [String: SettingValue] = [
    "SWIFT_INCLUDE_PATHS": "$(SRCROOT)/Dependencies/DictorFFI",
    "HEADER_SEARCH_PATHS": "$(SRCROOT)/Dependencies/DictorFFI",
    "LIBRARY_SEARCH_PATHS": "$(SRCROOT)/Dependencies/DictorFFI",
    // Expose dictorFFI.modulemap so Swift can import RustBuffer, ForeignBytes, etc.
    "OTHER_SWIFT_FLAGS": [
        "$(inherited)",
        "-Xcc",
        "-fmodule-map-file=$(SRCROOT)/Dependencies/DictorFFI/dictorFFI.modulemap",
    ],
    "OTHER_LDFLAGS": [
        "-ldictor",
        "-framework", "SystemConfiguration",
        "-framework", "Security",
        "-framework", "CoreFoundation"
    ]
]

// MARK: - Project

let project = Project(
    name: "Dictor",
    organizationName: "Dictor",
    targets: [

        // ═══════════════════════════════════════════════
        // MARK: - DictorApp (macOS Application)
        // ═══════════════════════════════════════════════

        .target(
            name: "DictorApp",
            destinations: .macOS,
            product: .app,
            bundleId: "com.dictor.app",
            deploymentTargets: .macOS("14.0"),
            infoPlist: .extendingDefault(with: [
                "CFBundleDisplayName": "Dictor",
                "LSApplicationCategoryType": "public.app-category.productivity",
                "NSMicrophoneUsageDescription": "Dictor needs microphone access to record audio for speech recognition.",
                "LSUIElement": .boolean(true),
            ]),
            sources: ["DictorApp/Sources/**"],
            resources: ["DictorApp/Resources/**"],
            dependencies: [
                .target(name: "DictorCore"),
                .external(name: "KeyboardShortcuts"),
            ],
            settings: .settings(
                base: sharedFFISettings.merging([
                    "CODE_SIGN_ENTITLEMENTS": "DictorApp/DictorApp.entitlements",
                    "ENABLE_TESTABILITY": "YES",
                ]) { current, _ in current }
            )
        ),

        // ═══════════════════════════════════════════════
        // MARK: - DictorCore (Framework — business logic)
        // ═══════════════════════════════════════════════

        .target(
            name: "DictorCore",
            destinations: .macOS,
            product: .framework,
            bundleId: "com.dictor.core",
            deploymentTargets: .macOS("14.0"),
            sources: [
                "DictorCore/Sources/**",
                "Dependencies/DictorFFI/dictor.swift"
            ],
            dependencies: [],
            settings: .settings(base: sharedFFISettings.merging([
                "ENABLE_TESTABILITY": "YES",
            ]) { current, _ in current })
        ),

        // ═══════════════════════════════════════════════
        // MARK: - Tests
        // ═══════════════════════════════════════════════

        .target(
            name: "DictorCoreTests",
            destinations: .macOS,
            product: .unitTests,
            bundleId: "com.dictor.core.tests",
            deploymentTargets: .macOS("14.0"),
            sources: ["DictorCore/Tests/**"],
            dependencies: [
                .target(name: "DictorCore"),
            ],
            settings: .settings(base: sharedFFISettings)
        ),

        .target(
            name: "DictorAppTests",
            destinations: .macOS,
            product: .unitTests,
            bundleId: "com.dictor.app.tests",
            deploymentTargets: .macOS("14.0"),
            sources: ["DictorApp/Tests/**"],
            dependencies: [
                .target(name: "DictorApp"),
                .target(name: "DictorCore"),
            ],
            settings: .settings(base: sharedFFISettings)
        ),
    ]
)
