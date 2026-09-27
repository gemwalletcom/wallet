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
                SecretDataTypeView(type: type(screen))
            }
            .cleanListRow()

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
        .detectScreenshots(docsUrl: docsUrl)
        .protectFromScreenRecording()
    }

    private var screen: GemSecretScreen {
        switch secret {
        case let .words(words): secretScreen(kind: .phrase, wordCount: UInt32(words.count), isNew: continueAction != nil)
        case .privateKey: secretScreen(kind: .privateKey, wordCount: 0, isNew: continueAction != nil)
        }
    }

    private func type(_ screen: GemSecretScreen) -> SecretPhraseDataType {
        switch secret {
        case let .words(words): .words(rows: screen.rows.map { $0.map(words: words) })
        case let .privateKey(key): .privateKey(key: key)
        }
    }

    private var secretCopy: GemCopy {
        switch secret {
        case let .words(words): secretPhraseCopy(words: words)
        case let .privateKey(key): privateKeyCopy(key: key)
        }
    }

    private var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }

    private func copy() {
        isPresentingCopyToast = true
    }

    private func onContinue() {
        continueAction?()
    }
}
