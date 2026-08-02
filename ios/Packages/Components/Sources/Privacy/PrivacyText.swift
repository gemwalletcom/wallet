// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct PrivacyText: View {
    public static let placeholder = "∗∗∗∗∗"

    @Binding private var isEnabled: Bool

    private let text: String
    private let placeholder: String

    public init(
        _ text: String,
        isEnabled: Binding<Bool>,
        placeholder: String = PrivacyText.placeholder,
    ) {
        self.text = text
        self.placeholder = placeholder
        _isEnabled = isEnabled
    }

    public var body: some View {
        Text(displayText)
    }

    private var displayText: String {
        isEnabled ? placeholder : text
    }
}
