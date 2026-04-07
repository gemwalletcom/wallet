// Copyright (c). Gem Wallet. All rights reserved.

import Localization
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit
import Validators

struct ConfirmNetworkFeeViewModelTests {
    @Test
    func loaded() {
        let fiatValue = "$2.50"
        let model = ConfirmNetworkFeeViewModel(
            state: .data(.mock()),
            title: Localized.Transfer.networkFee,
            value: "0.001 ETH",
            fiatValue: fiatValue,
            selectable: true,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == fiatValue)
        #expect(selectable == true)
    }

    @Test
    func loadedWithoutFiat() {
        let value = "0.001 ETH"
        let model = ConfirmNetworkFeeViewModel(
            state: .data(.mock()),
            title: Localized.Transfer.networkFee,
            value: value,
            fiatValue: nil,
            selectable: true,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == value)
        #expect(selectable == true)
    }

    @Test
    func error() {
        let model = ConfirmNetworkFeeViewModel(
            state: .error(AnyError("test")),
            title: Localized.Transfer.networkFee,
            value: nil,
            fiatValue: nil,
            selectable: true,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == "-")
        #expect(selectable == false)
    }

    @Test
    func calculatorError() {
        let value = "0.001 ETH"
        let fiatValue = "$2.50"
        let input = TransactionInputViewModel(
            data: .mock(),
            transactionData: .mock(),
            metaData: nil,
            transferAmount: .failure(.insufficientBalance(.mock())),
        )
        let model = ConfirmNetworkFeeViewModel(
            state: .data(input),
            title: Localized.Transfer.networkFee,
            value: value,
            fiatValue: fiatValue,
            selectable: true,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == value)
        #expect(item.subtitleExtra == fiatValue)
        #expect(selectable == true)
    }
}
