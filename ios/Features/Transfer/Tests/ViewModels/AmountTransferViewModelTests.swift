// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import class Gemstone.GemTransferService
import BigInt
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct AmountTransferViewModelTests {
    private let transferService = GemTransferService()

    @Test
    func title() {
        #expect(AmountTransferViewModel(asset: .mock(), action: .send(.mock()), amountService: GemAmountService()).title == "Send")
        #expect(AmountTransferViewModel(asset: .mock(), action: .deposit(.mock()), amountService: GemAmountService()).title == "Deposit")
        #expect(AmountTransferViewModel(asset: .mock(), action: .withdraw(.mock()), amountService: GemAmountService()).title == "Withdraw")
    }

    @Test
    func minimumValue() {
        let usdc = Asset.mock(symbol: "USDC")

        #expect(AmountTransferViewModel(asset: .mock(), action: .send(.mock()), amountService: GemAmountService()).minimumValue == .zero)
        #expect(AmountTransferViewModel(asset: usdc, action: .deposit(.mock()), amountService: GemAmountService()).minimumValue == PerpetualConfig.minDeposit)
        #expect(AmountTransferViewModel(asset: usdc, action: .withdraw(.mock()), amountService: GemAmountService()).minimumValue == PerpetualConfig.minWithdraw)
    }

    @Test
    func displayAsset() {
        let usdc = Asset.mock(symbol: "USDC")

        #expect(AmountTransferViewModel(asset: usdc, action: .send(.mock()), amountService: GemAmountService()).displayAsset.id == usdc.id)
        #expect(AmountTransferViewModel(asset: usdc, action: .deposit(.mock()), amountService: GemAmountService()).displayAsset.id == usdc.id)

        let withdraw = AmountTransferViewModel(asset: usdc, action: .withdraw(.mock()), amountService: GemAmountService()).displayAsset
        #expect(withdraw.id.identifier == PerpetualConfig.depositAssetId)
        #expect(withdraw.type == .token)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 1000, withdrawable: 500))

        #expect(AmountTransferViewModel(asset: .mock(), action: .send(.mock()), amountService: GemAmountService()).availableValue(from: assetData) == 1000)
        #expect(AmountTransferViewModel(asset: .mock(), action: .deposit(.mock()), amountService: GemAmountService()).availableValue(from: assetData) == 1000)
        #expect(AmountTransferViewModel(asset: .mock(), action: .withdraw(.mock()), amountService: GemAmountService()).availableValue(from: assetData) == 500)
    }

    @Test
    func recipientData() {
        let recipient = RecipientData.mock(recipient: .mock(address: "0x123"))
        #expect(AmountTransferViewModel(asset: .mock(), action: .send(recipient), amountService: GemAmountService()).recipientData().recipient.address == "0x123")
    }

    @Test
    func makeTransferData() throws {
        let send = try AmountTransferViewModel(asset: .mock(), action: .send(.mock()), amountService: GemAmountService()).makeTransferData(value: 100, useMaxAmount: false)
        let deposit = try AmountTransferViewModel(asset: .mock(), action: .deposit(.mock()), amountService: GemAmountService()).makeTransferData(value: 200, useMaxAmount: false)
        let withdraw = try AmountTransferViewModel(asset: .mock(), action: .withdraw(.mock()), amountService: GemAmountService()).makeTransferData(value: 300, useMaxAmount: false)

        #expect(send.inputType.transactionType(transferService: transferService) == .transfer)
        #expect(deposit.inputType.transactionType(transferService: transferService) == .transfer)
        #expect(withdraw.inputType.transactionType(transferService: transferService) == .transfer)
        #expect(send.value == "100")
        #expect(deposit.value == "200")
        #expect(withdraw.value == "300")
    }
}
