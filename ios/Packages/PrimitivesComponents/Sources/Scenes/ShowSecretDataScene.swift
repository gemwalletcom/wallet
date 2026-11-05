// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCopy
import struct Gemstone.GemSecretScreen
import enum Gemstone.GemWalletSecret
import func Gemstone.privateKeyCopy
import func Gemstone.secretPhraseCopy
import func Gemstone.secretScreen
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct ShowSecretDataScene: View {
    private let secret: GemWalletSecret
    private let continueAction: VoidAction
    @State private var isPresentingCopyToast = false

    public init(secret: GemWalletSecret, continueAction: VoidAction = nil) {
        self.secret = secret
        self.continueAction = continueAction
    }

    public var body: some View {
        let screen = screen
        List {
            Section {
                CalloutView(style: screen.warning.calloutViewStyle)
            }
            .cleanListRow()

            Section {
                SecretDataTypeView(secret: secret)
            }
            .cleanListRow()

            if let scope {
                Text(scope)
                    .textStyle(.calloutSecondary)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: .infinity)
                    .cleanListRow()
            }

            ListButton(
                title: Localized.Common.copy,
                image: Images.System.copy,
                action: copy,
            )
            .frame(maxWidth: .infinity, alignment: .center)
            .cleanListRow()
        }
        .safeAreaButton(isVisible: continueAction != nil) {
            StateButton(
                text: Localized.Common.continue,
                action: onContinue,
            )
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .toolbarInfoButton(url: docsUrl)
        .navigationTitle(screen.title.text)
        .copyToast(
            copy: secretCopy,
            isPresenting: $isPresentingCopyToast,
        )
        .detectScreenshots(kind: secret.kind)
        .protectFromScreenRecording()
    }

    private var screen: GemSecretScreen {
        secretScreen(kind: secret.kind, isNew: continueAction != nil)
    }

    private var scope: String? {
        switch secret {
        case .words: nil
        case let .privateKey(_, scope): scope.description
        }
    }

    private var secretCopy: GemCopy {
        switch secret {
        case let .words(words): secretPhraseCopy(words: words)
        case let .privateKey(key, _): privateKeyCopy(key: key)
        }
    }

    private var docsUrl: URL {
        secret.kind.securityDocsUrl
    }

    private func copy() {
        isPresentingCopyToast = true
    }

    private func onContinue() {
        continueAction?()
    }
}
