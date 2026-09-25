// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct AmountSceneViewModelTests {
    @Test
    func maxButton() {
        let model = AmountSceneViewModel.mock()
        #expect(model.amountInputModel.error == nil)

        model.onSelectMaxButton()
        #expect(model.amountInputModel.error == nil)
        #expect(model.entry.isMax)

        model.onSelectInputButton()
        #expect(model.amountInputType == .fiat)
        model.onSelectMaxButton()
        #expect(model.amountInputType == .asset)
        #expect(model.entry.isMax)
        #expect(model.amountInputModel.error == nil)
    }

    @Test
    func fiatInputConvertsWithThePrice() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 5_000_000_000_000_000_000),
            price: .mock(price: 2.5),
        )
        let model = AmountSceneViewModel.mock(assetData: assetData)

        model.onSelectInputButton()
        model.amountInputModel.text = "10"
        model.onChangeAmountText("", "10")

        #expect(model.entry.value == BigInt(4_000_000_000_000_000_000))
        #expect(model.amountInputModel.error == nil)
        #expect(!model.entry.isMax)
    }

    @Test
    func stakingReservedFeesText() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 2_000_000_000_000_000_000),
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.stake(validators: [DelegationValidator.mock().toGem()], validator: nil)),
            assetData: assetData,
        )

        model.onSelectMaxButton()
        #expect(model.infoText != nil)
        #expect(model.amountInputModel.text == "1.99975")

        model.amountInputModel.text = .zero
        model.onChangeAmountText("", .zero)
        #expect(model.infoText == nil)
    }

    @Test
    func unfreezeResourceSwitch() {
        let assetData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(frozen: 0, locked: 5_000_000),
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.unfreeze(resource: Resource.bandwidth.toGem())),
            assetData: assetData,
        )

        guard let stake = model.stake,
              case let .resource(resourceSelection) = stake.selection else { return }

        resourceSelection.selected = .energy
        model.onChangeResource(.bandwidth, .energy)
        model.amountInputModel.text = "2.0"
        model.onChangeAmountText("", "2.0")
        #expect(model.amountInputModel.error == nil)

        resourceSelection.selected = .bandwidth
        model.onChangeResource(.energy, .bandwidth)
        model.amountInputModel.text = "2.0"
        model.onChangeAmountText("", "2.0")
        #expect(model.amountInputModel.error != nil)
    }

    @Test
    func selectValidatorPreservesAmount() {
        let validator1 = DelegationValidator.mock(id: "1")
        let validator2 = DelegationValidator.mock(id: "2")
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 5_000_000_000_000_000_000),
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.stake(validators: [validator1.toGem(), validator2.toGem()], validator: nil)),
            assetData: assetData,
        )

        model.amountInputModel.text = "1.5"
        model.onChangeAmountText("", "1.5")
        model.onValidatorSelected(.mock(validator: validator2.toGem()))

        #expect(model.amountInputModel.text == "1.5")
    }

    @Test
    func actionButtonState() {
        let model = AmountSceneViewModel.mock()

        #expect(model.actionButtonState == .disabled)

        model.amountInputModel.text = "1.0"
        model.onChangeAmountText("", "1.0")
        #expect(model.actionButtonState == .normal)

        model.amountInputModel.text = ""
        model.onChangeAmountText("", "")
        #expect(model.actionButtonState == .disabled)
    }

    @Test
    func aFixedValueFillsItself() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: 1_000_000))
        let assetData = AssetData.mock(asset: .mockBNB())
        let model = AmountSceneViewModel.mock(
            type: .stake(.withdraw(delegation: delegation.toGem())),
            assetData: assetData,
        )

        #expect(model.isInputDisabled == true)

        model.prefillAmount()
        #expect(model.amountInputModel.text.isEmpty == false)
        #expect(model.input.focusesInput == false)
    }

    @Test
    func buyWithoutAccountDoesNotPresentSheet() {
        let assetData = AssetData.mock(asset: .mockBNB())
        let model = AmountSceneViewModel.mock(assetData: assetData)
        model.onSelectBuy()
        #expect(model.isPresentingSheet == nil)

        let withAccount = AmountSceneViewModel.mock(
            wallet: Wallet.mock(accounts: [.mock(chain: .smartChain)]),
            assetData: assetData,
        )
        withAccount.onSelectBuy()
        #expect(withAccount.isPresentingSheet != nil)
    }
}
