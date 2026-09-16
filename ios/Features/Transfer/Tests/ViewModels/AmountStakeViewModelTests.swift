// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import class Gemstone.GemAmountService
import enum Gemstone.GemStakeAmountInput
import struct Gemstone.GemValidatorRow
import GemstoneServicesTestKit
import Testing
@testable import Transfer

struct AmountStakeViewModelTests {
    @Test
    func title() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [DelegationValidator.mock().toGem()], validator: nil), service: GemAmountService.mock()).title == "Stake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation: Delegation.mock().toGem()), service: GemAmountService.mock()).title == "Unstake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()], delegation: Delegation.mock(validator: .mock(id: "from")).toGem(), validator: nil), service: GemAmountService.mock()).title == "Redelegate")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(delegation: Delegation.mock().toGem()), service: GemAmountService.mock()).title == "Withdraw")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem()), service: GemAmountService.mock()).title == "Freeze")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem()), service: GemAmountService.mock()).title == "Unfreeze")
    }

    @Test
    func validatorSelectionEnabled() {
        #expect(validatorState(.stake(validators: [DelegationValidator.mock().toGem()], validator: nil))?.isEnabled == true)
        #expect(validatorState(.unstake(delegation: Delegation.mock().toGem()))?.isEnabled == false)
        #expect(validatorState(.redelegate(validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()], delegation: Delegation.mock(validator: .mock(id: "from")).toGem(), validator: nil))?.isEnabled == true)
        #expect(validatorState(.withdraw(delegation: Delegation.mock().toGem()))?.isEnabled == false)
    }

    @Test
    func validatorSelection() {
        let first = DelegationValidator.mock(id: "first")
        let second = DelegationValidator.mock(id: "second")

        #expect(validatorState(.stake(validators: [first.toGem(), second.toGem()], validator: nil))?.selected.validator.id == "first")
        #expect(validatorState(.stake(validators: [first.toGem(), second.toGem()], validator: second.toGem()))?.selected.validator.id == "second")
    }

    @Test
    func resourceSelection() {
        let model = AmountStakeViewModel(asset: .mockTron(), type: .freeze(resource: Resource.energy.toGem()), service: GemAmountService.mock())
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
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [DelegationValidator.mock().toGem()], validator: nil), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(validators: [DelegationValidator.mock(id: "from").toGem(), DelegationValidator.mock(id: "to").toGem()], delegation: Delegation.mock(validator: .mock(id: "from")).toGem(), validator: nil), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(delegation: Delegation.mock().toGem()), service: GemAmountService.mock()).input(from: assetData).canChangeValue == false)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem()), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem()), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
    }

    @Test
    func availableValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: 5_000_000))
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 1000))

        let stake = AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [DelegationValidator.mock().toGem()], validator: nil), service: GemAmountService.mock())
        let unstake = AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation: delegation.toGem()), service: GemAmountService.mock())

        #expect(stake.input(from: assetData).availableValue == 1000)
        #expect(unstake.input(from: assetData).availableValue == 5_000_000)
    }

    @Test
    func availableValueForFreezeUnfreeze() {
        let tronData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(available: 1000, frozen: 2000, locked: 3000),
        )
        let freeze = AmountStakeViewModel(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem()), service: GemAmountService.mock())
        let unfreezeBandwidth = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(resource: Resource.bandwidth.toGem()), service: GemAmountService.mock())
        let unfreezeEnergy = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(resource: Resource.energy.toGem()), service: GemAmountService.mock())

        #expect(freeze.input(from: tronData).availableValue == 1000)
        #expect(unfreezeBandwidth.input(from: tronData).availableValue == 2000)
        #expect(unfreezeEnergy.input(from: tronData).availableValue == 3000)
    }

    @Test
    func makeTransferData() throws {
        let validator = DelegationValidator.mock(id: "validator1")
        let delegation = Delegation.mock(validator: validator)

        let stake = try AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [validator.toGem()], validator: nil), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let unstake = try AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation: delegation.toGem()), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let redelegate = try AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(validators: [validator.toGem(), DelegationValidator.mock(id: "validator2").toGem()], delegation: delegation.toGem(), validator: nil), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let withdraw = try AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(delegation: delegation.toGem()), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let freeze = try AmountStakeViewModel(asset: .mockTron(), type: .freeze(resource: Resource.bandwidth.toGem()), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let unfreeze = try AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(resource: Resource.energy.toGem()), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)

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

private func validatorState(_ type: GemStakeAmountInput, asset: Asset = .mockBNB()) -> SelectionState<GemValidatorRow>? {
    let model = AmountStakeViewModel(asset: asset, type: type, service: GemAmountService.mock())
    if case let .validator(state) = model.selection { return state }
    return nil
}
