// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemAmountService
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct AmountSceneRequestTests {
    @Test
    func aTransferNamesItsKind() {
        #expect(AmountSceneViewModel.mock().title == "Send")
        #expect(AmountSceneViewModel.mock(type: .deposit).title == "Deposit")
        #expect(AmountSceneViewModel.mock(type: .withdraw).title == "Withdraw")
    }

    @Test
    func aWithdrawShowsTheAssetItPaysOut() {
        let usdc = AssetData.mock(asset: .mock(symbol: "USDC"))

        #expect(AmountSceneViewModel.mock(assetData: usdc).displayAsset.id == usdc.asset.id)
        #expect(AmountSceneViewModel.mock(type: .deposit, assetData: usdc).displayAsset.id == usdc.asset.id)

        let withdraw = AmountSceneViewModel.mock(type: .withdraw, assetData: usdc).displayAsset
        #expect(withdraw.id.identifier == "arbitrum_0xaf88d065e77c8cC2239327C5EDb3A432268e5831")
        #expect(withdraw.type == .erc20)
    }

    @Test
    func aWithdrawSpendsTheWithdrawableBalance() {
        let assetData = AssetData.mock(balance: .mock(available: 1000, withdrawable: 500))

        #expect(AmountSceneViewModel.mock(assetData: assetData).input.availableValue == 1000)
        #expect(AmountSceneViewModel.mock(type: .deposit, assetData: assetData).input.availableValue == 1000)
        #expect(AmountSceneViewModel.mock(type: .withdraw, assetData: assetData).input.availableValue == 500)
    }

    @Test
    func aPaymentAmountPrefillsTheInput() {
        let recipient = GemPaymentRecipient.mock(recipient: .mock(address: "0x123"), amount: "1.5")
        let assetData = AssetData.mock(asset: .mock(decimals: 6))

        #expect(AmountSceneViewModel.mock(type: .transfer(recipient: recipient), assetData: assetData).input.prefill?.value == 1_500_000)
        #expect(AmountSceneViewModel.mock(type: .deposit, assetData: assetData).input.prefill == nil)
    }

    @Test
    func anEarnNamesItsProvider() {
        let deposit = AmountSceneViewModel.mock(type: .earn(.deposit(DelegationValidator.mock(name: "Figment").toGem())))
        let withdraw = AmountSceneViewModel.mock(type: .earn(.withdraw(Delegation.mock(validator: .mock(name: "Chorus One")).toGem())))

        #expect(deposit.earnProviderRow?.name == "Figment")
        #expect(withdraw.earnProviderRow?.name == "Chorus One")
        #expect(deposit.title != withdraw.title)
        #expect(AmountSceneViewModel.mock().earnProviderRow == nil)
    }

    @Test
    func theRequestBuildsTheTransferItNames() async throws {
        let send = try await transferData(AmountSceneViewModel.mock(), value: 100, useMaxAmount: false)
        let deposit = try await transferData(AmountSceneViewModel.mock(type: .deposit), value: 200, useMaxAmount: true)

        guard case .transfer = send.inputType else {
            Issue.record("expected a transfer, got \(send.inputType)")
            return
        }
        guard case .deposit = deposit.inputType else {
            Issue.record("expected a deposit, got \(deposit.inputType)")
            return
        }
        #expect(send.value == "100")
        #expect(deposit.value == "200")
        #expect(deposit.useMaxAmount)
    }

    private func transferData(_ model: AmountSceneViewModel, value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await GemAmountService.mock().transferData(asset: model.asset.toGem(), request: model.request, value: value, useMaxAmount: useMaxAmount)
    }
}
