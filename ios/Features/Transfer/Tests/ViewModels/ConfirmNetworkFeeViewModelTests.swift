// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
@testable import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit
import Validators

struct ConfirmNetworkFeeViewModelTests {
    @Test
    func loaded() {
        let feeModel = feeModel(feeAssetPrice: Price(price: 2500, priceChangePercentage24h: 0, updatedAt: Date()))
        let model = ConfirmNetworkFeeViewModel(
            state: .data(.mock()),
            feeModel: feeModel,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == feeModel.fiatValue)
        #expect(item.subtitle != feeModel.value)
        #expect(selectable == true)
    }

    @Test
    func loadedWithoutFiat() {
        let feeModel = feeModel()
        let model = ConfirmNetworkFeeViewModel(
            state: .data(.mock()),
            feeModel: feeModel,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(feeModel.fiatValue == nil)
        #expect(item.subtitle == feeModel.value)
        #expect(selectable == true)
    }

    @Test
    func error() {
        let model = ConfirmNetworkFeeViewModel(
            state: .error(AnyError("test")),
            feeModel: feeModel(feeAmount: nil),
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == "-")
        #expect(selectable == false)
    }

    @Test
    func calculatorError() {
        let input = ConfirmTransferInput.mock(transferAmount: .failure(.insufficientBalance(
            .mock(),
            requirement: BalanceRequirement(required: 1, available: 0),
        )))
        let feeModel = feeModel(feeAssetPrice: Price(price: 2500, priceChangePercentage24h: 0, updatedAt: Date()))
        let model = ConfirmNetworkFeeViewModel(
            state: .data(input),
            feeModel: feeModel,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == feeModel.fiatValue)
        #expect(item.subtitleExtra == nil)
        #expect(selectable == true)
    }

    private func feeModel(
        feeAssetPrice: Price? = nil,
        feeAmount: BigInt? = BigInt(1_000_000_000_000_000),
    ) -> NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: .mockEthereum(),
            currency: .usd,
            selection: .preset(.normal),
            feeAssetPrice: feeAssetPrice,
            feeAmount: feeAmount,
        )
    }
}
