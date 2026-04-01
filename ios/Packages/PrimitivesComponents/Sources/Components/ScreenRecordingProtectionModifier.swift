// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

struct ScreenRecordingProtectionModifier: ViewModifier {
    @Environment(\.scenePhase) private var scenePhase
    @State private var isRecording = false

    func body(content: Content) -> some View {
        ZStack {
            content
                .opacity(isRecording ? 0 : 1)

            if isRecording {
                Text(Localized.SecretPhrase.ContentHidden.description)
                    .font(.callout)
                    .foregroundStyle(Colors.secondaryText)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .onChange(of: scenePhase) { _, _ in
            updateRecordingState()
        }
        .onAppear {
            updateRecordingState()
        }
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

public extension View {
    func protectFromScreenRecording() -> some View {
        modifier(ScreenRecordingProtectionModifier())
    }
}
