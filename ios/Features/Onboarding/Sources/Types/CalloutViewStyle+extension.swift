// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Style

extension CalloutViewStyle {
    static func secretPhraseWarning() -> CalloutViewStyle {
        secretWarning(title: Localized.SecretPhrase.DoNotShare.title, subtitle: Localized.SecretPhrase.DoNotShare.description)
    }

    static func privateKeyWarning(chainName: String) -> CalloutViewStyle {
        secretWarning(title: Localized.PrivateKey.DoNotShare.title, subtitle: Localized.PrivateKey.DoNotShare.description(chainName))
    }

    private static func secretWarning(title: String, subtitle: String) -> CalloutViewStyle {
        CalloutViewStyle(
            title: TextValue(
                text: title,
                style: TextStyle(font: .system(.body, weight: .medium), color: Colors.red),
            ),
            subtitle: TextValue(
                text: subtitle,
                style: TextStyle(font: .callout, color: Colors.red),
            ),
            backgroundColor: Colors.redLight,
        )
    }

    static func header(title: String) -> CalloutViewStyle {
        CalloutViewStyle(
            title: TextValue(
                text: title,
                style: TextStyle(font: .app.body, color: Colors.secondaryText),
            ),
            subtitle: nil,
            backgroundColor: .clear,
        )
    }
}
