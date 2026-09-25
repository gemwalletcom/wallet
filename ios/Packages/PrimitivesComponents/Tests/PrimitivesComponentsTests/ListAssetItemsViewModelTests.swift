// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemAssetItemRow
import enum Gemstone.GemSelectAssetType
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct ListAssetItemsViewModelTests {
    @Test
    func receiveCollectionUsesNetworkName() {
        let ton = Asset.mock(
            id: AssetId(chain: .ton, tokenId: nil),
            name: "Gram",
            symbol: "GRAM",
            decimals: 9,
            type: .native,
        )

        #expect(row(.mock(asset: ton), .receiveCollection).title == "TON")
        #expect(row(.mock(asset: ton), .receive).title == "Gram")
    }

    @Test
    func theSymbolShowsOnlyWhenItAddsToTheTitleTheRowShows() {
        let usdc = AssetData.mock(asset: .mock(id: AssetId(chain: .ethereum, tokenId: "0xusdc"), name: "USDC", symbol: "USDC", decimals: 6, type: .erc20))
        let ethereum = AssetData.mock(asset: .mockEthereum())

        #expect(row(usdc, .receive).titleExtra == nil, "a name that already is the symbol is not repeated")
        #expect(row(ethereum, .receive).titleExtra == "ETH")
        #expect(row(ethereum, .send).titleExtra == nil, "the send list shows no symbol at all")
    }

    @Test
    func theBalanceAndItsFiatAreTheOnesCoreFormatted() {
        let held = row(.mock(asset: .mockEthereum(), balance: .mock(available: BigInt("2000000000000000000")), price: .mock(price: 1500)), .send)
        let empty = row(.mock(asset: .mockEthereum(), balance: .zero, price: .mock(price: 1500)), .send)

        guard case let .value(balance, fiat) = held.trailing, case let .value(_, emptyFiat) = empty.trailing else {
            Issue.record("a send row trails its balance")
            return
        }
        #expect(balance.text.text == "2 ETH")
        #expect(fiat?.text.text == "$3,000.00")
        #expect(emptyFiat == nil, "an empty balance is worth nothing the row can name")
    }

    @Test
    func theNetworkSubtitleNamesTheChainForTokensOnly() {
        #expect(row(.mock(asset: .mockEthereumUSDT()), .receive).subtitle?.text.text == "Ethereum")
        #expect(row(.mock(asset: .mockEthereum()), .receive).subtitle == nil, "a coin's row already names its network")
    }

    @Test
    func theWalletListHidesItsBalancesWithThePrivacyToggle() {
        let wallet = ListAssetItemsViewModel(currency: .usd).rows([.mock(asset: .mockEthereum(), price: .mock(price: 1500))])

        #expect(wallet.first?.masksBalance == true)
    }

    private func row(_ assetData: AssetData, _ type: GemSelectAssetType) -> GemAssetItemRow {
        ListAssetItemsViewModel(currency: .usd, rowStyle: type.flow().rowStyle).rows([assetData])[0]
    }
}
