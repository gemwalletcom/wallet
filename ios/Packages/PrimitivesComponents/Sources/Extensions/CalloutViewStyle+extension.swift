// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Style

extension CalloutViewStyle {
    static func secretDataWarning(title: String, description: String) -> CalloutViewStyle {
        CalloutViewStyle(
            title: TextValue(
                text: title,
                style: TextStyle(font: .system(.body, weight: .medium), color: Colors.red),
            ),
            subtitle: TextValue(
                text: description,
                style: TextStyle(font: .callout, color: Colors.red),
            ),
            backgroundColor: Colors.redLight,
        )
    }

    public static func header(title: String) -> CalloutViewStyle {
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
