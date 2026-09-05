import struct Gemstone.GemHeaderButton
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct WalletHeaderViewModelTests {
    @Test
    func title() {
        let model = WalletHeaderViewModel(
            walletType: .multicoin,
            totalValue: .mock(value: 1000),
            currencyCode: Currency.usd.rawValue,
            showsPnl: true,
            buttons: [],
        )
        #expect(model.title == "$1,000.00")
    }

    @Test
    func titleSmallValue() {
        let model = WalletHeaderViewModel(
            walletType: .multicoin,
            totalValue: .mock(value: 0.1041, pnlAmount: 0),
            currencyCode: Currency.usd.rawValue,
            showsPnl: true,
            buttons: [],
        )
        #expect(model.title == "$0.10")
    }

    @Test
    func subtitle() {
        let model = WalletHeaderViewModel(
            walletType: .multicoin,
            totalValue: .mock(value: 1000, pnlAmount: 50, pnlPercentage: 5),
            currencyCode: Currency.usd.rawValue,
            showsPnl: true,
            buttons: [],
        )
        #expect(model.subtitle == "+$50.00 (5.00%)")
    }

    @Test
    func subtitleSmallPnlAmount() {
        let model = WalletHeaderViewModel(
            walletType: .multicoin,
            totalValue: .mock(value: 61.40, pnlAmount: 0.1041, pnlPercentage: 0.17),
            currencyCode: Currency.usd.rawValue,
            showsPnl: true,
            buttons: [],
        )
        #expect(model.subtitle == "+$0.10 (0.17%)")
    }

    @Test
    func buttonsDisabled() {
        let model = WalletHeaderViewModel(
            walletType: .multicoin,
            totalValue: .mock(),
            currencyCode: Currency.usd.rawValue,
            showsPnl: true,
            buttons: [GemHeaderButton(kind: .send, isEnabled: false), GemHeaderButton(kind: .swap, isEnabled: false)],
        )
        #expect(model.buttons.allSatisfy { !$0.isEnabled })
    }
}
