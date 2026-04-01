// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Style

extension CalloutViewStyle {
    static func secretDataWarning() -> CalloutViewStyle {
        CalloutViewStyle(
            title: TextValue(
                text: Localized.SecretPhrase.DoNotShare.title,
                style: TextStyle(font: .system(.body, weight: .medium), color: Colors.red),
            ),
            subtitle: TextValue(
                text: Localized.SecretPhrase.DoNotShare.description,
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
