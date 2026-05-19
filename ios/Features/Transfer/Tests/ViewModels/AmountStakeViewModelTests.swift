// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct AmountStakeViewModelTests {
    @Test
    func title() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil)).title == "Stake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock())).title == "Unstake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil)).title == "Redelegate")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock())).title == "Withdraw")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth)).title == "Freeze")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth)).title == "Unfreeze")
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
        let model = AmountStakeViewModel(asset: .mockTron(), type: .freeze(.energy))
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
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil)).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil)).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock())).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock())).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth)).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth)).validatorSelectType == .unstake)
    }

    @Test
    func canChangeValue() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil)).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil)).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock())).canChangeValue == false)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth)).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth)).canChangeValue == true)
    }

    @Test
    func reserveForFee() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil)).reserveForFee > .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .stake(validators: [.mock()], recommended: nil)).reserveForFee == .zero)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock())).reserveForFee == .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth)).reserveForFee > .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth)).reserveForFee == .zero)
    }

    @Test
    func minimumValue() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil)).minimumValue > .zero)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock())).minimumValue == .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth)).minimumValue > .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth)).minimumValue == .zero)
    }

    @Test
    func availableValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "5000000"))
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 1000))

        let stake = AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil))
        let unstake = AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation))

        #expect(stake.availableValue(from: assetData) == 1000)
        #expect(unstake.availableValue(from: assetData) == 5_000_000)
    }

    @Test
    func availableValueForFreezeUnfreeze() {
        let tronData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(available: 1000, frozen: 2000, locked: 3000),
        )
        let freeze = AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth))
        let unfreezeBandwidth = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth))
        let unfreezeEnergy = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.energy))

        #expect(freeze.availableValue(from: tronData) == 1000)
        #expect(unfreezeBandwidth.availableValue(from: tronData) == 2000)
        #expect(unfreezeEnergy.availableValue(from: tronData) == 3000)
    }

    @Test
    func shouldReserveFee() {
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 5_000_000_000_000_000_000))
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "1000000"))
        let unstake = AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation))

        #expect(unstake.shouldReserveFee(from: assetData) == false)
    }

    @Test
    func makeTransferData() throws {
        let validator = DelegationValidator.mock(id: "validator1")
        let delegation = Delegation.mock(validator: validator)

        let stake = try AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [validator], recommended: nil)).makeTransferData(value: 100)
        let unstake = try AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation)).makeTransferData(value: 100)
        let redelegate = try AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(delegation, validators: [validator], recommended: nil)).makeTransferData(value: 100)
        let withdraw = try AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(delegation)).makeTransferData(value: 100)
        let freeze = try AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth)).makeTransferData(value: 100)
        let unfreeze = try AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.energy)).makeTransferData(value: 100)

        #expect(stake.type.transactionType == .stakeDelegate)
        #expect(unstake.type.transactionType == .stakeUndelegate)
        #expect(redelegate.type.transactionType == .stakeRedelegate)
        #expect(withdraw.type.transactionType == .stakeWithdraw)
        #expect(freeze.type.transactionType == .stakeFreeze)
        #expect(unfreeze.type.transactionType == .stakeUnfreeze)
        #expect(stake.value == 100)
        #expect(unstake.value == 100)
        #expect(redelegate.value == 100)
        #expect(withdraw.value == 100)
        #expect(freeze.value == 100)
        #expect(unfreeze.value == 100)
    }
}

private func validatorState(_ type: AmountStakeType, asset: Asset = .mockBNB()) -> SelectionState<DelegationValidator>? {
    let model = AmountStakeViewModel(asset: asset, type: type)
    if case let .validator(state) = model.selection { return state }
    return nil
}
