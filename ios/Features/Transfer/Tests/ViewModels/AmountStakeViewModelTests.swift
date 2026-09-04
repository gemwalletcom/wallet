// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import class Gemstone.GemAmountService
import GemstoneServicesTestKit
import Testing
@testable import Transfer

struct AmountStakeViewModelTests {
    @Test
    func title() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), service: GemAmountService.mock()).title == "Stake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock()), service: GemAmountService.mock()).title == "Unstake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil), service: GemAmountService.mock()).title == "Redelegate")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock()), service: GemAmountService.mock()).title == "Withdraw")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), service: GemAmountService.mock()).title == "Freeze")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), service: GemAmountService.mock()).title == "Unfreeze")
    }

    @Test
    func validatorSelectionEnabled() {
        #expect(validatorState(.stake(validators: [.mock()], recommended: nil))?.isEnabled == true)
        #expect(validatorState(.unstake(.mock()))?.isEnabled == false)
        #expect(validatorState(.redelegate(.mock(), validators: [.mock()], recommended: nil))?.isEnabled == true)
        #expect(validatorState(.withdraw(.mock()))?.isEnabled == false)
    }

    @Test
    func validatorSelection() {
        let recommended = DelegationValidator.mock(id: "recommended")
        let first = DelegationValidator.mock(id: "first")
        let second = DelegationValidator.mock(id: "second")

        #expect(validatorState(.stake(validators: [first, recommended], recommended: recommended))?.selected.id == "recommended")
        #expect(validatorState(.stake(validators: [first, second], recommended: nil))?.selected.id == "first")
    }

    @Test
    func resourceSelection() {
        let model = AmountStakeViewModel(asset: .mockTron(), type: .freeze(.energy), service: GemAmountService.mock())
        guard case let .resource(state) = model.selection else {
            Issue.record("Expected resource selection")
            return
        }
        #expect(state.options == [.bandwidth, .energy])
        #expect(state.selected == .energy)
        #expect(state.isEnabled == true)
    }

    @Test
    func validatorSelectType() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), service: GemAmountService.mock()).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil), service: GemAmountService.mock()).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock()), service: GemAmountService.mock()).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock()), service: GemAmountService.mock()).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), service: GemAmountService.mock()).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), service: GemAmountService.mock()).validatorSelectType == .unstake)
    }

    @Test
    func canChangeValue() {
        let assetData = AssetData.mock(asset: .mockBNB())
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock()), service: GemAmountService.mock()).input(from: assetData).canChangeValue == false)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), service: GemAmountService.mock()).input(from: assetData).canChangeValue == true)
    }

    @Test
    func availableValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "5000000"))
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 1000))

        let stake = AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), service: GemAmountService.mock())
        let unstake = AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation), service: GemAmountService.mock())

        #expect(stake.input(from: assetData).availableValue == 1000)
        #expect(unstake.input(from: assetData).availableValue == 5_000_000)
    }

    @Test
    func availableValueForFreezeUnfreeze() {
        let tronData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(available: 1000, frozen: 2000, locked: 3000),
        )
        let freeze = AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), service: GemAmountService.mock())
        let unfreezeBandwidth = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), service: GemAmountService.mock())
        let unfreezeEnergy = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.energy), service: GemAmountService.mock())

        #expect(freeze.input(from: tronData).availableValue == 1000)
        #expect(unfreezeBandwidth.input(from: tronData).availableValue == 2000)
        #expect(unfreezeEnergy.input(from: tronData).availableValue == 3000)
    }

    @Test
    func makeTransferData() throws {
        let validator = DelegationValidator.mock(id: "validator1")
        let delegation = Delegation.mock(validator: validator)

        let stake = try AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [validator], recommended: nil), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let unstake = try AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let redelegate = try AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(delegation, validators: [validator], recommended: nil), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let withdraw = try AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(delegation), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let freeze = try AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)
        let unfreeze = try AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.energy), service: GemAmountService.mock()).makeTransferData(value: 100, useMaxAmount: false)

        #expect(stake.inputType.transactionType().map() == .stakeDelegate)
        #expect(unstake.inputType.transactionType().map() == .stakeUndelegate)
        #expect(redelegate.inputType.transactionType().map() == .stakeRedelegate)
        #expect(withdraw.inputType.transactionType().map() == .stakeWithdraw)
        #expect(freeze.inputType.transactionType().map() == .stakeFreeze)
        #expect(unfreeze.inputType.transactionType().map() == .stakeUnfreeze)
        #expect(stake.value == "100")
        #expect(unstake.value == "100")
        #expect(redelegate.value == "100")
        #expect(withdraw.value == "100")
        #expect(freeze.value == "100")
        #expect(unfreeze.value == "100")
    }
}

private func validatorState(_ type: AmountStakeType, asset: Asset = .mockBNB()) -> SelectionState<DelegationValidator>? {
    let model = AmountStakeViewModel(asset: asset, type: type, service: GemAmountService.mock())
    if case let .validator(state) = model.selection { return state }
    return nil
}
