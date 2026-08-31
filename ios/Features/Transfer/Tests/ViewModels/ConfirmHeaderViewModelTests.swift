// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
@testable import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmHeaderViewModelTests {
    @Test
    func amountShowsClearHeader() {
        let headerType = TransactionHeaderType.amount(
            .numeric(
                NumericViewModel(
                    data: AssetValuePrice(asset: .mockEthereumUSDT(), value: BigInt(1), price: nil),
                    style: AmountDisplayStyle(currencyCode: "USD"),
                ),
            ),
        )
        #expect(headerType.showsClearHeader == true)
    }

    @Test
    func swapHidesClearHeader() {
        let headerType = TransactionHeaderType.swap(
            from: SwapAmountField(
                assetId: .mockEthereum(),
                assetImage: AssetImage(),
                amount: "1 ETH",
                fiatAmount: "$1",
            ),
            to: SwapAmountField(
                assetId: Asset.mockEthereumUSDT().id,
                assetImage: AssetImage(),
                amount: "2 USDC",
                fiatAmount: "$2",
            ),
        )
        #expect(headerType.showsClearHeader == false)
    }

    @Test
    func nftShowsClearHeader() {
        #expect(TransactionHeaderType.nft(name: nil, image: AssetImage()).showsClearHeader == true)
    }

    @Test
    func assetShowsClearHeader() {
        #expect(TransactionHeaderType.asset(image: AssetImage()).showsClearHeader == true)
    }

    @Test
    func simulationHeaderDataResolvesAssetValue() {
        let model = ConfirmHeaderViewModel(
            request: .mock(),
            state: .mock(simulation: .mock(headerData: AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .exact(BigInt(1_000_000))))),
            currency: .usd,
        )

        guard case let .header(item) = model.itemModel else { return }
        guard case let .assetValue(data) = item.headerType else {
            Issue.record("Expected assetValue header")
            return
        }
        #expect(data.asset == .mockEthereumUSDT())
        #expect(data.value == .exact(BigInt(1_000_000)))
        #expect(item.showClearHeader == true)
    }

    @Test
    func tokenApproveResolvesAssetHeader() {
        let model = ConfirmHeaderViewModel(
            request: .mock(data: .mock(type: .tokenApprove(.mock(), .mock()))),
            state: .mock(),
            currency: .usd,
        )

        guard case let .header(item) = model.itemModel else { return }
        guard case .asset = item.headerType else {
            Issue.record("Expected asset header")
            return
        }
        #expect(item.showClearHeader == true)
    }
}
