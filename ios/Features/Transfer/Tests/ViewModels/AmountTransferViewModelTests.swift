// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemPaymentRecipient
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct AmountTransferViewModelTests {
    @Test
    func title() {
        #expect(AmountTransferViewModel.mock().title == "Send")
        #expect(AmountTransferViewModel.mock(transfer: .deposit).title == "Deposit")
        #expect(AmountTransferViewModel.mock(transfer: .withdraw).title == "Withdraw")
    }

    @Test
    func displayAsset() {
        let usdc = Asset.mock(symbol: "USDC")

        #expect(AmountTransferViewModel.mock(asset: usdc).displayAsset.id == usdc.id)
        #expect(AmountTransferViewModel.mock(asset: usdc, transfer: .deposit).displayAsset.id == usdc.id)

        let withdraw = AmountTransferViewModel.mock(asset: usdc, transfer: .withdraw).displayAsset
        #expect(withdraw.id.identifier == "arbitrum_0xaf88d065e77c8cC2239327C5EDb3A432268e5831")
        #expect(withdraw.type == .erc20)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(balance: .mock(available: 1000, withdrawable: 500))

        #expect(AmountTransferViewModel.mock().input(from: assetData).availableValue == 1000)
        #expect(AmountTransferViewModel.mock(transfer: .deposit).input(from: assetData).availableValue == 1000)
        #expect(AmountTransferViewModel.mock(transfer: .withdraw).input(from: assetData).availableValue == 500)
    }

    @Test
    func prefilledAmount() {
        let recipient = GemPaymentRecipient.mock(recipient: .mock(address: "0x123"), amount: "1.5")
        #expect(AmountTransferViewModel.mock(transfer: .send(payment: recipient)).prefilledAmount == "1.5")
        #expect(AmountTransferViewModel.mock(transfer: .deposit).prefilledAmount == nil)
    }

    @Test
    func makeTransferData() async throws {
        let send = try await AmountTransferViewModel.mock().makeTransferData(value: 100, useMaxAmount: false)
        let deposit = try await AmountTransferViewModel.mock(transfer: .deposit).makeTransferData(value: 200, useMaxAmount: true)

        #expect({
            if case .transfer = send.inputType {
                true
            } else {
                false
            }
        }())
        #expect({
            if case .deposit = deposit.inputType {
                true
            } else {
                false
            }
        }())
        #expect(send.value == "100")
        #expect(deposit.value == "200")
        #expect(deposit.useMaxAmount)
    }
}
