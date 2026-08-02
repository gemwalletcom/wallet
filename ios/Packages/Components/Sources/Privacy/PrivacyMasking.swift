// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Style
import SwiftUI

public extension String {
    func masked(if hideBalance: Bool) -> String {
        hideBalance ? PrivacyText.placeholder : self
    }
}

public extension TextValue {
    func masked(if hideBalance: Bool) -> TextValue {
        guard hideBalance else { return self }
        return TextValue(text: PrivacyText.placeholder, style: style, lineLimit: lineLimit, truncationMode: truncationMode)
    }
}

public extension Optional where Wrapped == TextValue {
    func masked(if hideBalance: Bool) -> TextValue? {
        self?.masked(if: hideBalance)
    }
}
