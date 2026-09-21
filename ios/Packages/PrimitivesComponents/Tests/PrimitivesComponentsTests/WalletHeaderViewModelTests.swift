import func Gemstone.formattedCurrency
import func Gemstone.formattedPercentage
import func Gemstone.formattedSignedCurrency
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemHeaderActions
import struct Gemstone.GemHeaderButton
import enum Gemstone.GemLocalizedText
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct WalletHeaderViewModelTests {
    @Test
    func title() {
        #expect(model(total: 1000).title == "$1,000.00")
    }

    @Test
    func titleSmallValue() {
        #expect(model(total: 0.1041).title == "$0.10")
    }

    @Test
    func subtitle() {
        #expect(model(total: 1000, pnlAmount: 50, pnlPercentage: 5).subtitle == "+$50.00 (5.00%)")
    }

    @Test
    func subtitleSmallPnlAmount() {
        #expect(model(total: 61.40, pnlAmount: 0.1041, pnlPercentage: 0.17).subtitle == "+$0.10 (0.17%)")
    }

    @Test
    func noChangeShowsNoSubtitle() {
        #expect(model(total: 1000).subtitle == nil)
    }

    @Test
    func buttonsDisabled() {
        let model = WalletHeaderViewModel(
            total: currency(0),
            pnl: nil,
            pnlTone: .plain,
            actions: .buttons(buttons: [GemHeaderButton(kind: .send, isEnabled: false), GemHeaderButton(kind: .swap, isEnabled: false)]),
        )
        #expect(model.buttons.allSatisfy { !$0.isEnabled })
    }

    private func model(total: Double, pnlAmount: Double? = nil, pnlPercentage: Double = 0) -> WalletHeaderViewModel {
        WalletHeaderViewModel(
            total: currency(total),
            pnl: pnlAmount.map {
                .pnl(
                    amount: formattedSignedCurrency(value: $0, code: Currency.usd.rawValue, style: .fiat),
                    percent: formattedPercentage(value: pnlPercentage, style: .unsigned),
                )
            },
            pnlTone: .positive,
            actions: .buttons(buttons: []),
        )
    }

    private func currency(_ value: Double) -> GemFormattedNumber {
        formattedCurrency(value: value, code: Currency.usd.rawValue, style: .fiat)
    }
}
