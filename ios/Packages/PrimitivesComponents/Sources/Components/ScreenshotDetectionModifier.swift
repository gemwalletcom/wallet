// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletSecretKind
import Localization
import SwiftUI

struct ScreenshotDetectionModifier: ViewModifier {
    let kind: GemWalletSecretKind

    @State private var showingAlert = false

    func body(content: Content) -> some View {
        content
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.userDidTakeScreenshotNotification)) { _ in
                showingAlert = true
            }
            .alert(
                Localized.SecretPhrase.ScreenshotDetected.title,
                isPresented: $showingAlert,
            ) {
                Button(Localized.Common.learnMore) {
                    UIApplication.shared.open(kind.securityDocsUrl)
                }
                Button(Localized.Common.done, role: .cancel) {}
            } message: {
                Text(kind.screenshotDetectedDescription)
            }
    }
}

public extension View {
    @ViewBuilder
    func detectScreenshots(kind: GemWalletSecretKind?) -> some View {
        if let kind {
            modifier(ScreenshotDetectionModifier(kind: kind))
        } else {
            self
        }
    }
}
