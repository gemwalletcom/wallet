// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

struct ScreenRecordingProtectionModifier: ViewModifier {
    @Environment(\.scenePhase) private var scenePhase
    @State private var isRecording = false
    @State private var captureStateRegistration: (any UITraitChangeRegistration)?

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
        .onAppear {
            guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene else { return }
            isRecording = windowScene.traitCollection.sceneCaptureState == .active
            captureStateRegistration = windowScene.registerForTraitChanges([UITraitSceneCaptureState.self]) { (windowScene: UIWindowScene, _) in
                isRecording = windowScene.traitCollection.sceneCaptureState == .active
            }
        }
        .onDisappear {
            guard let captureStateRegistration else { return }
            (UIApplication.shared.connectedScenes.first as? UIWindowScene)?.unregisterForTraitChanges(captureStateRegistration)
        }
        .excludeFromWindowCapture()
    }
}

public extension View {
    func protectFromScreenRecording() -> some View {
        modifier(ScreenRecordingProtectionModifier())
    }
}
