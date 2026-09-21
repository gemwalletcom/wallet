// Copyright (c). Gem Wallet. All rights reserved.

@testable import Components
import SwiftUI
import Testing

@MainActor
struct CurrencyTextFieldTests {
    private func height(of text: String) -> CGFloat {
        let view = CurrencyTextField(
            "0",
            text: .constant(text),
            currencySymbol: "$",
            currencyPosition: .leading,
            keyboardType: .decimalPad,
        )
        return UIHostingController(rootView: view)
            .sizeThatFits(in: CGSize(width: 320, height: UIView.layoutFittingCompressedSize.height))
            .height
    }

    @Test
    func heightDoesNotChangeWithTheTypedAmount() {
        let empty = height(of: "")

        #expect(height(of: "0") == empty)
        #expect(height(of: "1234567890") == empty)
        #expect(height(of: "1,234.56") == empty)
    }
}
