// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.addressCopy
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemRowMenuItem
import func Gemstone.walletRow
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
@testable import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Style
import Testing

struct GemListRowItemTests {
    @Test
    func latencyRowsRenderMeasurementsLoadingAndErrors() {
        let row = GemListRow.latency(title: .stream, titleSuffix: "", host: "api.gemwallet.com", status: .result(latency: .init(latencyType: .fast, value: 125)))
        guard case let .listItem(model) = row.item(onInfo: nil) else {
            Issue.record("Expected a latency row")
            return
        }
        #expect(model.title == "Stream")
        #expect(model.titleExtra == "api.gemwallet.com")
        #expect(model.titleTag == Localized.Common.latencyInMs(125))
        #expect(model.titleTagStyle.color == Colors.green)

        guard case let .listItem(loading) = GemListRow.latency(title: .api, titleSuffix: "", host: "api.gemwallet.com", status: .loading).item(onInfo: nil),
              case .progressView = loading.titleTagType
        else {
            Issue.record("Expected a loading badge")
            return
        }
        #expect(loading.title == "API")

        guard case let .listItem(error) = GemListRow.latency(title: .gemWalletNode, titleSuffix: " 🇺🇸", host: "gemnodes.com", status: .error).item(onInfo: nil) else {
            Issue.record("Expected an error badge")
            return
        }
        #expect(error.title == Localized.Nodes.gemWalletNode + " 🇺🇸")
        #expect(error.titleTag == Localized.Errors.error)
        #expect(error.titleTagStyle.color == Colors.red)
    }

    @Test
    func aPendingStatusSpinsInItsTone() {
        let row = GemListRow.label(title: .status, text: .transactionState(state: .pending), tone: .warning, info: nil, progress: true)
        guard case let .listItem(model) = row.item(onInfo: nil), case .progressView = model.subtitleTagType else {
            Issue.record("Expected a spinning status row")
            return
        }
        #expect(model.title == Localized.Transaction.status)
        #expect(model.subtitle == Localized.Transaction.Status.pending)
        #expect(model.subtitleStyle.color == Colors.orange)
    }

    @Test
    func aNetworkRowShowsTheNameCoreGives() {
        guard case let .network(title, subtitle, _) = GemListRow.network(title: .network, chain: Chain.ethereum.rawValue, name: "Ethereum (ERC20)").item(onInfo: nil) else {
            Issue.record("Expected a network row")
            return
        }
        #expect(title == Localized.Transfer.network)
        #expect(subtitle == "Ethereum (ERC20)")
    }

    @Test
    func aQuoteRowKeepsTheChangeBesideThePrice() {
        let price = GemFormattedNumber.mock(value: 2.55, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest)
        let change = GemFormattedNumber.mock(value: -0.69, unit: .percent, display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .negative, rounding: .toNearest)
        guard case let .listItem(model) = GemListRow.quote(title: .price, value: price, change: change).item(onInfo: nil) else {
            Issue.record("Expected a quote row")
            return
        }
        #expect(model.title == Localized.Asset.price)
        #expect(model.subtitle == price.text())
        #expect(model.subtitleSuffix == change.text())
        #expect(model.subtitleSuffixStyle.color == Colors.red)
        #expect(model.subtitleExtra == nil)
    }

    @Test
    func anAppRowCarriesItsWebsiteMenu() {
        let website: GemRowMenuItem = .open(title: .rowTitle(title: .website), url: "https://pancakeswap.finance")
        guard case let .imageMenu(model, menu) = GemListRow.app(title: .app, name: "PancakeSwap", iconUrl: nil, menu: [website]).item(onInfo: nil) else {
            Issue.record("Expected an app row")
            return
        }
        #expect(model.title == Localized.WalletConnect.app)
        #expect(model.subtitle == "PancakeSwap")
        #expect(menu == [website])
    }

    @Test
    func aWalletRowCarriesItsMenu() {
        let wallet = Wallet.mock()
        let menu: [GemRowMenuItem] = [
            .copy(copy: addressCopy(chain: Chain.ethereum.rawValue, address: "0x1")),
            .open(title: .viewOn(name: "Etherscan"), url: "https://etherscan.io/address/0x1"),
        ]
        guard case let .imageMenu(model, rowMenu) = GemListRow.wallet(title: .wallet, wallet: walletRow(wallet: wallet.toGem()), menu: menu).item(onInfo: nil) else {
            Issue.record("Expected a wallet row")
            return
        }
        #expect(model.title == Localized.Common.wallet)
        #expect(model.subtitle == wallet.name)
        #expect(model.imageStyle != nil)
        #expect(rowMenu == menu)
    }

    @Test
    func aMemoRowReadsItsTitleFromCore() {
        guard case let .menu(model, menu) = GemListRow.memo(title: .memo, value: "12345", menu: []).item(onInfo: nil) else {
            Issue.record("Expected a memo row")
            return
        }
        #expect(model.title == Localized.Transfer.memo)
        #expect(model.subtitle == "12345")
        #expect(menu.isEmpty)
    }

    @Test
    func anInfoTopicBecomesTheRowsInfoAction() {
        var opened: GemInfoTopic?
        let row = GemListRow.label(title: .status, text: .transactionState(state: .confirmed), tone: .positive, info: .stakeApr(chain: Primitives.Chain.tron.rawValue), progress: false)
        guard case let .listItem(model) = row.item(onInfo: { opened = $0 }) else {
            Issue.record("Expected a list item")
            return
        }
        model.infoAction?()
        #expect(opened == .stakeApr(chain: Primitives.Chain.tron.rawValue))
    }

    @Test
    func aPositionRowJoinsItsPnlAndMargin() {
        let pnl = GemListRow.label(
            title: .pnl,
            text: .pnl(
                amount: .mock(value: 500, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest),
                percent: .mock(value: 50, unit: .percent, display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest),
            ),
            tone: .positive,
            info: nil,
            progress: false,
        )
        let margin = GemListRow.label(
            title: .margin,
            text: .margin(amount: .mock(value: 1000, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest), marginType: .isolated),
            tone: .plain,
            info: nil,
            progress: false,
        )
        guard case let .listItem(pnlModel) = pnl.item(onInfo: nil), case let .listItem(marginModel) = margin.item(onInfo: nil) else {
            Issue.record("Expected list items")
            return
        }
        #expect(pnlModel.subtitle == "+$500.00 (+50.00%)")
        #expect(marginModel.subtitle == "$1,000.00 (Isolated)")
    }

    @Test
    func marketCapCarriesItsRankTag() {
        guard case let .listItem(model) = GemListRow.ranked(
            title: .marketCap,
            amount: .mock(value: 1_000_000, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest),
            tag: "#7",
        ).item(onInfo: nil) else {
            Issue.record("Expected a list item")
            return
        }
        #expect(model.title == Localized.Asset.marketCap)
        #expect(model.titleTag == " #7 ")
    }

    @Test
    func anAllTimeRowShowsItsDateAndTonedChange() {
        let row = GemListRow.allTime(
            title: .allTimeHigh,
            value: .mock(value: 100, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest),
            date: Date(),
            change: .mock(value: -12, unit: .percent, display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .negative, rounding: .toNearest),
        )
        guard case let .listItem(model) = row.item(onInfo: nil) else {
            Issue.record("Expected a list item")
            return
        }
        #expect(model.title == Localized.Asset.allTimeHigh)
        #expect(model.titleExtra != nil)
        #expect(model.subtitle == "$100.00")
        #expect(model.subtitleExtra == "-12.00%")
        #expect(model.subtitleStyleExtra.color == Colors.red)
    }

    @Test
    func anIdentifierOpensTheExplorerOnlyWhenCoreGivesALink() {
        let copy = addressCopy(chain: Chain.ethereum.rawValue, address: "0xdAC17F958D2ee523a2206206994597C13D831ec7")
        let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/token/0xdAC17F958D2ee523a2206206994597C13D831ec7")
        let menu: [GemRowMenuItem] = [.copy(copy: copy)]
        guard case let .explorerPage(model, url, _) = GemListRow.identifier(title: .contract, copy: copy, explorer: link.toGem(), address: copy.value, menu: menu).item(onInfo: nil),
              case let .menu(plain, plainMenu) = GemListRow.identifier(title: .tokenId, copy: copy, explorer: nil, address: nil, menu: menu).item(onInfo: nil)
        else {
            Issue.record("Expected an explorer page and a copyable row")
            return
        }
        #expect(model.title == Localized.Asset.contract)
        #expect(model.subtitle == copy.display)
        #expect(url == link.url)
        #expect(plain.title == Localized.Asset.tokenId)
        #expect(plain.subtitle == copy.display)
        #expect(plainMenu == menu)
    }

    @Test
    func autocloseLinesOfferTheirExplanation() {
        var opened: GemInfoTopic?
        let row = GemListRow.lines(title: .autoClose, lines: [.text(text: "TP: $120.00")], info: .autoClose)
        guard case let .listItem(model) = row.item(onInfo: { opened = $0 }) else {
            Issue.record("Expected a list item")
            return
        }
        model.infoAction?()
        #expect(model.subtitle == "TP: $120.00")
        #expect(opened == .autoClose)
    }
}
