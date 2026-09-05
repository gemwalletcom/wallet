// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemPaymentRecipient
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct AmountTransferViewModelTests {
    @Test
    func title() {
        #expect(AmountTransferViewModel(asset: .mock(), action: .send(.mock())).title == "Send")
        #expect(AmountTransferViewModel(asset: .mock(), action: .deposit(.mock())).title == "Deposit")
        #expect(AmountTransferViewModel(asset: .mock(), action: .withdraw(.mock())).title == "Withdraw")
    }

    @Test
    func displayAsset() {
        let usdc = Asset.mock(symbol: "USDC")

        #expect(AmountTransferViewModel(asset: usdc, action: .send(.mock())).displayAsset.id == usdc.id)
        #expect(AmountTransferViewModel(asset: usdc, action: .deposit(.mock())).displayAsset.id == usdc.id)

        let withdraw = AmountTransferViewModel(asset: usdc, action: .withdraw(.mock())).displayAsset
        #expect(withdraw.id.identifier == PerpetualConfig.depositAssetId)
        #expect(withdraw.type == .token)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 1000, withdrawable: 500))

        #expect(AmountTransferViewModel(asset: .mock(), action: .send(.mock())).input(from: assetData).availableValue == 1000)
        #expect(AmountTransferViewModel(asset: .mock(), action: .deposit(.mock())).input(from: assetData).availableValue == 1000)
        #expect(AmountTransferViewModel(asset: .mock(), action: .withdraw(.mock())).input(from: assetData).availableValue == 500)
    }

    @Test
    func recipientData() {
        let recipient = GemPaymentRecipient(recipient: .mock(address: "0x123"))
        #expect(AmountTransferViewModel(asset: .mock(), action: .send(recipient)).recipientData().recipient.address == "0x123")
    }

    @Test
    func makeTransferData() throws {
        let send = try AmountTransferViewModel(asset: .mock(), action: .send(.mock())).makeTransferData(value: 100, useMaxAmount: false)
        let deposit = try AmountTransferViewModel(asset: .mock(), action: .deposit(.mock())).makeTransferData(value: 200, useMaxAmount: false)
        let withdraw = try AmountTransferViewModel(asset: .mock(), action: .withdraw(.mock())).makeTransferData(value: 300, useMaxAmount: false)

        #expect(send.inputType.transactionType().map() == .transfer)
        #expect(deposit.inputType.transactionType().map() == .transfer)
        #expect(withdraw.inputType.transactionType().map() == .transfer)
        #expect(send.value == "100")
        #expect(deposit.value == "200")
        #expect(withdraw.value == "300")
    }
}
