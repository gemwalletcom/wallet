// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemAmountService
import struct Gemstone.GemAmountInput
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemTransferData
import struct Gemstone.GemValidatorRow
import enum Gemstone.StakeType
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import GemstoneServicesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct AmountStakeViewModelTests {
    @Test
    func title() {
        #expect(amountTitle(AmountStakeViewModel.mock()) == "Stake")
        #expect(amountTitle(AmountStakeViewModel.mock(type: .unstake(delegation: Delegation.mock().toGem()))) == "Unstake")
        #expect(amountTitle(AmountStakeViewModel.mock(type: .redelegate(
            validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()],
            delegation: Delegation.mock(validator: .mock(id: "from")).toGem(),
            validator: nil,
        ))) == "Redelegate")
        #expect(amountTitle(AmountStakeViewModel.mock(type: .withdraw(delegation: Delegation.mock().toGem()))) == "Withdraw")
        #expect(amountTitle(AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem()))) == "Freeze")
        #expect(amountTitle(AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem()))) == "Unfreeze")
    }

    @Test
    func validatorSelectionEnabled() {
        #expect(AmountStakeViewModel.mock().validatorState?.isEnabled == true)
        #expect(AmountStakeViewModel.mock(type: .unstake(delegation: Delegation.mock().toGem())).validatorState?.isEnabled == false)
        #expect(AmountStakeViewModel.mock(type: .redelegate(
            validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()],
            delegation: Delegation.mock(validator: .mock(id: "from")).toGem(),
            validator: nil,
        )).validatorState?.isEnabled == true)
        #expect(AmountStakeViewModel.mock(type: .withdraw(delegation: Delegation.mock().toGem())).validatorState?.isEnabled == false)
    }

    @Test
    func validatorSelection() {
        let first = DelegationValidator.mock(id: "first")
        let second = DelegationValidator.mock(id: "second")

        #expect(AmountStakeViewModel.mock(type: .stake(validators: [first.toGem(), second.toGem()], validator: nil)).validatorState?.selected.validator.id == "first")
        #expect(AmountStakeViewModel.mock(type: .stake(validators: [first.toGem(), second.toGem()], validator: second.toGem())).validatorState?.selected.validator.id == "second")
    }

    @Test
    func resourceSelection() {
        let model = AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.energy.toGem()))
        guard case let .resource(state) = model.selection else {
            Issue.record("Expected resource selection")
            return
        }
        #expect(state.options == [.bandwidth, .energy])
        #expect(state.selected == .energy)
        #expect(state.isEnabled == true)
    }

    @Test
    func canChangeValue() {
        let assetData = AssetData.mock(asset: .mockBNB())
        #expect(amountInput(AmountStakeViewModel.mock(), assetData).canChangeValue == true)
        #expect(amountInput(AmountStakeViewModel.mock(type: .redelegate(
            validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()],
            delegation: Delegation.mock(validator: .mock(id: "from")).toGem(),
            validator: nil,
        )), assetData).canChangeValue == true)
        #expect(amountInput(AmountStakeViewModel.mock(type: .withdraw(delegation: Delegation.mock().toGem())), assetData).canChangeValue == false)
        #expect(amountInput(AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem())), assetData).canChangeValue == true)
        #expect(amountInput(AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem())), assetData).canChangeValue == true)
    }

    @Test
    func availableValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: 5_000_000))
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 1000))

        let stake = AmountStakeViewModel.mock()
        let unstake = AmountStakeViewModel.mock(type: .unstake(delegation: delegation.toGem()))

        #expect(amountInput(stake, assetData).availableValue == 1000)
        #expect(amountInput(unstake, assetData).availableValue == 5_000_000)
    }

    @Test
    func availableValueForFreezeUnfreeze() {
        let tronData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(available: 1000, frozen: 2000, locked: 3000),
        )
        let freeze = AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem()))
        let unfreezeBandwidth = AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem()))
        let unfreezeEnergy = AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.energy.toGem()))

        #expect(amountInput(freeze, tronData).availableValue == 1000)
        #expect(amountInput(unfreezeBandwidth, tronData).availableValue == 2000)
        #expect(amountInput(unfreezeEnergy, tronData).availableValue == 3000)
    }

    @Test
    func makeTransferData() async throws {
        let validator = DelegationValidator.mock(id: "validator1")
        let delegation = Delegation.mock(validator: validator)

        let stake = try await transferData(AmountStakeViewModel.mock(type: .stake(validators: [validator.toGem()], validator: nil)), value: 100)
        let unstake = try await transferData(AmountStakeViewModel.mock(type: .unstake(delegation: delegation.toGem())), value: 100)
        let redelegate = try await transferData(AmountStakeViewModel.mock(type: .redelegate(validators: [validator.toGem(), DelegationValidator.mock(id: "validator2").toGem()], delegation: delegation.toGem(), validator: nil)), value: 100)
        let withdraw = try await transferData(AmountStakeViewModel.mock(type: .withdraw(delegation: delegation.toGem())), value: 100)
        let freeze = try await transferData(AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem())), value: 100)
        let unfreeze = try await transferData(AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.energy.toGem())), value: 100)

        #expect(stakeType(stake) == .stake(validator.toGem()))
        #expect(stakeType(unstake) == .unstake(delegation.toGem()))
        #expect(stakeType(redelegate).map {
            if case .redelegate = $0 {
                true
            } else {
                false
            }
        } == true)
        #expect(stakeType(withdraw) == .withdraw(delegation.toGem()))
        #expect(stakeType(freeze) == .freeze(Resource.bandwidth.toGem()))
        #expect(stakeType(unfreeze) == .unfreeze(Resource.energy.toGem()))
        #expect(stake.value == "100")
        #expect(unstake.value == "100")
        #expect(redelegate.value == "100")
        #expect(withdraw.value == "100")
        #expect(freeze.value == "100")
        #expect(unfreeze.value == "100")
    }

    private func stakeType(_ data: GemTransferData) -> Gemstone.StakeType? {
        guard case let .stake(_, stakeType) = data.inputType else { return nil }
        return stakeType
    }

    private func amountTitle(_ model: AmountStakeViewModel) -> String {
        model.request.amountType().title().title
    }

    private func amountInput(_ model: AmountStakeViewModel, _ assetData: AssetData) -> GemAmountInput {
        model.request.input(asset: model.asset.toGem(), balance: GemAssetBalance(assetData.balance, assetId: model.asset.id, isActive: assetData.metadata.isActive))
    }

    private func transferData(_ model: AmountStakeViewModel, value: BigInt) async throws -> GemTransferData {
        try await GemAmountService.mock().transferData(asset: model.asset.toGem(), request: model.request, value: value, useMaxAmount: false)
    }
}

private extension AmountStakeViewModel {
    var validatorState: SelectionState<GemValidatorRow>? {
        if case let .validator(state) = selection {
            return state
        }
        return nil
    }
}
