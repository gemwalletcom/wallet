// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import class Gemstone.GemTransferService
import BigInt
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct AmountStakeViewModelTests {
    private let transferService = GemTransferService()

    @Test
    func title() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), amountService: GemAmountService()).title == "Stake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock()), amountService: GemAmountService()).title == "Unstake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil), amountService: GemAmountService()).title == "Redelegate")
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock()), amountService: GemAmountService()).title == "Withdraw")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), amountService: GemAmountService()).title == "Freeze")
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), amountService: GemAmountService()).title == "Unfreeze")
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
        let model = AmountStakeViewModel(asset: .mockTron(), type: .freeze(.energy), amountService: GemAmountService())
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
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), amountService: GemAmountService()).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil), amountService: GemAmountService()).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock()), amountService: GemAmountService()).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock()), amountService: GemAmountService()).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), amountService: GemAmountService()).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), amountService: GemAmountService()).validatorSelectType == .unstake)
    }

    @Test
    func canChangeValue() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), amountService: GemAmountService()).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(.mock(), validators: [.mock()], recommended: nil), amountService: GemAmountService()).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(.mock()), amountService: GemAmountService()).canChangeValue == false)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), amountService: GemAmountService()).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), amountService: GemAmountService()).canChangeValue == true)
    }

    @Test
    func reserveForFee() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), amountService: GemAmountService()).reserveForFee > .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .stake(validators: [.mock()], recommended: nil), amountService: GemAmountService()).reserveForFee == .zero)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock()), amountService: GemAmountService()).reserveForFee == .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), amountService: GemAmountService()).reserveForFee > .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), amountService: GemAmountService()).reserveForFee == .zero)
    }

    @Test
    func minimumValue() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), amountService: GemAmountService()).minimumValue > .zero)
        #expect(AmountStakeViewModel(asset: .mockBNB(), type: .unstake(.mock()), amountService: GemAmountService()).minimumValue == .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), amountService: GemAmountService()).minimumValue > .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), amountService: GemAmountService()).minimumValue == .zero)
    }

    @Test
    func availableValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "5000000"))
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 1000))

        let stake = AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [.mock()], recommended: nil), amountService: GemAmountService())
        let unstake = AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation), amountService: GemAmountService())

        #expect(stake.availableValue(from: assetData) == 1000)
        #expect(unstake.availableValue(from: assetData) == 5_000_000)
    }

    @Test
    func availableValueForFreezeUnfreeze() {
        let tronData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(available: 1000, frozen: 2000, locked: 3000),
        )
        let freeze = AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), amountService: GemAmountService())
        let unfreezeBandwidth = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.bandwidth), amountService: GemAmountService())
        let unfreezeEnergy = AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.energy), amountService: GemAmountService())

        #expect(freeze.availableValue(from: tronData) == 1000)
        #expect(unfreezeBandwidth.availableValue(from: tronData) == 2000)
        #expect(unfreezeEnergy.availableValue(from: tronData) == 3000)
    }

    @Test
    func shouldReserveFee() {
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 5_000_000_000_000_000_000))
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "1000000"))
        let unstake = AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation), amountService: GemAmountService())

        #expect(unstake.shouldReserveFee(from: assetData) == false)
    }

    @Test
    func makeTransferData() throws {
        let validator = DelegationValidator.mock(id: "validator1")
        let delegation = Delegation.mock(validator: validator)

        let stake = try AmountStakeViewModel(asset: .mockBNB(), type: .stake(validators: [validator], recommended: nil), amountService: GemAmountService()).makeTransferData(value: 100, useMaxAmount: false)
        let unstake = try AmountStakeViewModel(asset: .mockBNB(), type: .unstake(delegation), amountService: GemAmountService()).makeTransferData(value: 100, useMaxAmount: false)
        let redelegate = try AmountStakeViewModel(asset: .mockBNB(), type: .redelegate(delegation, validators: [validator], recommended: nil), amountService: GemAmountService()).makeTransferData(value: 100, useMaxAmount: false)
        let withdraw = try AmountStakeViewModel(asset: .mockBNB(), type: .withdraw(delegation), amountService: GemAmountService()).makeTransferData(value: 100, useMaxAmount: false)
        let freeze = try AmountStakeViewModel(asset: .mockTron(), type: .freeze(.bandwidth), amountService: GemAmountService()).makeTransferData(value: 100, useMaxAmount: false)
        let unfreeze = try AmountStakeViewModel(asset: .mockTron(), type: .unfreeze(.energy), amountService: GemAmountService()).makeTransferData(value: 100, useMaxAmount: false)

        #expect(stake.type.transactionType(transferService: transferService) == .stakeDelegate)
        #expect(unstake.type.transactionType(transferService: transferService) == .stakeUndelegate)
        #expect(redelegate.type.transactionType(transferService: transferService) == .stakeRedelegate)
        #expect(withdraw.type.transactionType(transferService: transferService) == .stakeWithdraw)
        #expect(freeze.type.transactionType(transferService: transferService) == .stakeFreeze)
        #expect(unfreeze.type.transactionType(transferService: transferService) == .stakeUnfreeze)
        #expect(stake.value == 100)
        #expect(unstake.value == 100)
        #expect(redelegate.value == 100)
        #expect(withdraw.value == 100)
        #expect(freeze.value == 100)
        #expect(unfreeze.value == 100)
    }
}

private func validatorState(_ type: AmountStakeType, asset: Asset = .mockBNB()) -> SelectionState<DelegationValidator>? {
    let model = AmountStakeViewModel(asset: asset, type: type, amountService: GemAmountService())
    if case let .validator(state) = model.selection { return state }
    return nil
}
