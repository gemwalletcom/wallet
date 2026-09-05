// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemPaymentRecipient
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import GemstoneServicesTestKit
import class Gemstone.GemAmountService
import Testing
@testable import Transfer

struct AmountTransferViewModelTests {
    @Test
    func title() {
        #expect(AmountTransferViewModel(asset: .mock(), action: .send(.mock()), service: GemAmountService.mock()).title == "Send")
        #expect(AmountTransferViewModel(asset: .mock(), action: .deposit, service: GemAmountService.mock()).title == "Deposit")
        #expect(AmountTransferViewModel(asset: .mock(), action: .withdraw, service: GemAmountService.mock()).title == "Withdraw")
    }

    @Test
    func displayAsset() {
        let usdc = Asset.mock(symbol: "USDC")

        #expect(AmountTransferViewModel(asset: usdc, action: .send(.mock()), service: GemAmountService.mock()).displayAsset.id == usdc.id)
        #expect(AmountTransferViewModel(asset: usdc, action: .deposit, service: GemAmountService.mock()).displayAsset.id == usdc.id)

        let withdraw = AmountTransferViewModel(asset: usdc, action: .withdraw, service: GemAmountService.mock()).displayAsset
        #expect(withdraw.id.identifier == PerpetualConfig.depositAssetId)
        #expect(withdraw.type == .token)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 1000, withdrawable: 500))

        #expect(AmountTransferViewModel(asset: .mock(), action: .send(.mock()), service: GemAmountService.mock()).input(from: assetData).availableValue == 1000)
        #expect(AmountTransferViewModel(asset: .mock(), action: .deposit, service: GemAmountService.mock()).input(from: assetData).availableValue == 1000)
        #expect(AmountTransferViewModel(asset: .mock(), action: .withdraw, service: GemAmountService.mock()).input(from: assetData).availableValue == 500)
    }

    @Test
    func prefilledAmount() {
        let recipient = GemPaymentRecipient(recipient: .mock(address: "0x123"), amount: "1.5")
        #expect(AmountTransferViewModel(asset: .mock(), action: .send(recipient), service: GemAmountService.mock()).prefilledAmount == "1.5")
        #expect(AmountTransferViewModel(asset: .mock(), action: .deposit, service: GemAmountService.mock()).prefilledAmount == nil)
    }

    @Test
    func makeTransferData() async throws {
        let send = try await AmountTransferViewModel(asset: .mock(), action: .send(.mock()), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let deposit = try await AmountTransferViewModel(asset: .mock(), action: .deposit, service: GemAmountService.mock()).makeTransferData(value: 200, useMaxAmount: true)

        #expect(send.inputType.transactionType().map() == .transfer)
        #expect(deposit.inputType.transactionType().map() == .transfer)
        #expect(send.value == "100")
        #expect(deposit.value == "200")
        #expect(deposit.useMaxAmount)
    }
}
