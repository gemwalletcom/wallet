// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

struct ScreenRecordingProtectionModifier: ViewModifier {
    @Environment(\.scenePhase) private var scenePhase
    @State private var isRecording = false

    private var isHidden: Bool {
        isRecording || scenePhase != .active
    }

    func body(content: Content) -> some View {
        ZStack {
            content
                .opacity(isHidden ? 0 : 1)
                .accessibilityHidden(isHidden)

            if isHidden {
                Text(Localized.SecretPhrase.ContentHidden.description)
                    .font(.callout)
                    .foregroundStyle(Colors.secondaryText)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .onChange(of: scenePhase) { _, _ in
            updateRecordingState()
        }
        .onReceive(NotificationCenter.default.publisher(for: UIScreen.capturedDidChangeNotification)) { _ in
            updateRecordingState()
        }
        .onAppear {
            updateRecordingState()
            #if targetEnvironment(macCatalyst)
                WindowCaptureExclusion.begin()
            #endif
        }
        #if targetEnvironment(macCatalyst)
        .onDisappear {
            WindowCaptureExclusion.end()
        }
        #endif
    }

    private func updateRecordingState() {
        guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
              let window = windowScene.windows.first
        else {
            return
        }

        isRecording = window.screen.traitCollection.sceneCaptureState == .active
    }
}

#if targetEnvironment(macCatalyst)
    /// On Mac the secure layer, the capture state and the screenshot notification are all unavailable,
    /// so while a protected screen is shown the app's windows are left out of screenshots, recordings and screen sharing.
    @MainActor
    private enum WindowCaptureExclusion {
        private static let sharingTypeNone = 0
        private static let sharingTypeReadOnly = 1
        private static var protectedScreens = 0

        static func begin() {
            protectedScreens += 1
            if protectedScreens == 1 {
                setSharingType(sharingTypeNone)
            }
        }

        static func end() {
            protectedScreens = max(protectedScreens - 1, 0)
            if protectedScreens == 0 {
                setSharingType(sharingTypeReadOnly)
            }
        }

        private static func setSharingType(_ sharingType: Int) {
            guard let applicationClass: AnyObject = NSClassFromString("NSApplication"),
                  let application = applicationClass.value(forKey: "sharedApplication") as? NSObject,
                  let windows = application.value(forKey: "windows") as? [NSObject]
            else {
                return
            }
            windows.forEach { $0.setValue(sharingType, forKey: "sharingType") }
        }
    }
#endif

public extension View {
    func protectFromScreenRecording() -> some View {
        modifier(ScreenRecordingProtectionModifier())
    }
}
