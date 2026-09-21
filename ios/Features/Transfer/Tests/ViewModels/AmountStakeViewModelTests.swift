// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct AmountStakeViewModelTests {
    @Test
    func title() {
        #expect(AmountStakeViewModel.mock().title == "Stake")
        #expect(AmountStakeViewModel.mock(type: .unstake(delegation: Delegation.mock().toGem())).title == "Unstake")
        #expect(AmountStakeViewModel.mock(type: .redelegate(
            validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()],
            delegation: Delegation.mock(validator: .mock(id: "from")).toGem(),
            validator: nil,
        )).title == "Redelegate")
        #expect(AmountStakeViewModel.mock(type: .withdraw(delegation: Delegation.mock().toGem())).title == "Withdraw")
        #expect(AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem())).title == "Freeze")
        #expect(AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem())).title == "Unfreeze")
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
        #expect(AmountStakeViewModel.mock().input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel.mock(type: .redelegate(
            validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()],
            delegation: Delegation.mock(validator: .mock(id: "from")).toGem(),
            validator: nil,
        )).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel.mock(type: .withdraw(delegation: Delegation.mock().toGem())).input(from: assetData).canChangeValue == false)
        #expect(AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem())).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem())).input(from: assetData).canChangeValue == true)
    }

    @Test
    func availableValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: 5_000_000))
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 1000))

        let stake = AmountStakeViewModel.mock()
        let unstake = AmountStakeViewModel.mock(type: .unstake(delegation: delegation.toGem()))

        #expect(stake.input(from: assetData).availableValue == 1000)
        #expect(unstake.input(from: assetData).availableValue == 5_000_000)
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

        #expect(freeze.input(from: tronData).availableValue == 1000)
        #expect(unfreezeBandwidth.input(from: tronData).availableValue == 2000)
        #expect(unfreezeEnergy.input(from: tronData).availableValue == 3000)
    }

    @Test
    func makeTransferData() throws {
        let validator = DelegationValidator.mock(id: "validator1")
        let delegation = Delegation.mock(validator: validator)

        let stake = try AmountStakeViewModel.mock(type: .stake(validators: [validator.toGem()], validator: nil)).makeTransferData(value: 100, useMaxAmount: false)
        let unstake = try AmountStakeViewModel.mock(type: .unstake(delegation: delegation.toGem())).makeTransferData(value: 100, useMaxAmount: false)
        let redelegate = try AmountStakeViewModel.mock(type: .redelegate(validators: [validator.toGem(), DelegationValidator.mock(id: "validator2").toGem()], delegation: delegation.toGem(), validator: nil)).makeTransferData(
            value: 100,
            useMaxAmount: false,
        )
        let withdraw = try AmountStakeViewModel.mock(type: .withdraw(delegation: delegation.toGem())).makeTransferData(value: 100, useMaxAmount: false)
        let freeze = try AmountStakeViewModel.mock(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem())).makeTransferData(value: 100, useMaxAmount: false)
        let unfreeze = try AmountStakeViewModel.mock(asset: .mockTron(), type: .unfreeze(resource: Resource.energy.toGem())).makeTransferData(value: 100, useMaxAmount: false)

        #expect(stake.transactionType().toPrimitives() == .stakeDelegate)
        #expect(unstake.transactionType().toPrimitives() == .stakeUndelegate)
        #expect(redelegate.transactionType().toPrimitives() == .stakeRedelegate)
        #expect(withdraw.transactionType().toPrimitives() == .stakeWithdraw)
        #expect(freeze.transactionType().toPrimitives() == .stakeFreeze)
        #expect(unfreeze.transactionType().toPrimitives() == .stakeUnfreeze)
        #expect(stake.value == "100")
        #expect(unstake.value == "100")
        #expect(redelegate.value == "100")
        #expect(withdraw.value == "100")
        #expect(freeze.value == "100")
        #expect(unfreeze.value == "100")
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
