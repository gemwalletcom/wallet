import func Gemstone.formattedCurrency
import func Gemstone.formattedPercentage
import struct Gemstone.GemFormattedNumber
import struct Gemstone.GemHeaderButton
import enum Gemstone.GemLocalizedText
import struct Gemstone.GemWalletHomeViewState
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct WalletHeaderTests {
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
        let model = GemWalletHomeViewState.mock(
            total: currency(0),
            headerActions: .buttons(buttons: [GemHeaderButton(kind: .send, isEnabled: false), GemHeaderButton(kind: .swap, isEnabled: false)]),
        ).valueHeader
        #expect(model.buttons.allSatisfy { !$0.isEnabled })
    }

    private func model(total: Double, pnlAmount: Double? = nil, pnlPercentage: Double = 0) -> ValueHeader {
        GemWalletHomeViewState.mock(
            total: currency(total),
            pnl: pnlAmount.map {
                .pnl(
                    amount: signed(currency($0)),
                    percent: formattedPercentage(value: pnlPercentage, style: .unsigned),
                )
            },
            pnlTone: .positive,
            headerActions: .buttons(buttons: []),
        ).valueHeader
    }

    private func currency(_ value: Double) -> GemFormattedNumber {
        formattedCurrency(value: value, code: Currency.usd.rawValue, style: .fiat)
    }

    private func signed(_ number: GemFormattedNumber) -> GemFormattedNumber {
        var signed = number
        signed.notation = .signed
        return signed
    }
}
