// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemAmountService
import struct Gemstone.GemPaymentRecipient
import enum Gemstone.GemStakeAmountInput
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing
@testable import Transfer

@MainActor
struct AmountSceneViewModelTests {
    @Test
    func maxButton() {
        let model = AmountSceneViewModel.mock()
        #expect(model.amountInputModel.isValid)

        model.onSelectMaxButton()
        #expect(model.amountInputModel.isValid)
        #expect(model.entry.isMax)

        model.onSelectInputButton()
        #expect(model.amountInputType == .fiat)
        model.onSelectMaxButton()
        #expect(model.amountInputType == .asset)
        #expect(model.entry.isMax)
        #expect(model.amountInputModel.isValid)
    }

    @Test
    func fiatInputConvertsWithThePrice() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 5_000_000_000_000_000_000),
            price: .mock(price: 2.5),
        )
        let model = AmountSceneViewModel.mock(
            type: .transfer(recipient: GemPaymentRecipient(recipient: .mock())),
            assetData: assetData,
        )

        model.onSelectInputButton()
        model.amountInputModel.text = "10"
        model.onChangeAmountText("", "10")

        #expect(model.entry.value == BigInt(4_000_000_000_000_000_000))
        #expect(model.amountInputModel.isValid)
        #expect(!model.entry.isMax)
    }

    @Test
    func stakingReservedFeesText() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 2_000_000_000_000_000_000),
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.stake(validators: [DelegationValidator.mock().map()], validator: nil)),
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
    func stakeValidation() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 5_000_000_000_000_000_000),
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.stake(validators: [DelegationValidator.mock().map()], validator: nil)),
            assetData: assetData,
        )

        model.amountInputModel.text = "0.099"
        model.onChangeAmountText("", "0.099")
        #expect(model.amountInputModel.isValid == false)

        model.amountInputModel.text = "1.5"
        model.onChangeAmountText("", "1.5")
        #expect(model.amountInputModel.isValid == true)
    }

    @Test
    func transferValidation() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 10_000_000_000_000_000),
        )
        let model = AmountSceneViewModel.mock(
            type: .transfer(recipient: GemPaymentRecipient(recipient: .mock())),
            assetData: assetData,
        )

        model.amountInputModel.text = "0.001"
        model.onChangeAmountText("", "0.001")
        #expect(model.amountInputModel.isValid == true)

        model.amountInputModel.text = "100"
        model.onChangeAmountText("", "100")
        #expect(model.amountInputModel.isValid == false)
    }

    @Test
    func unfreezeResourceSwitch() {
        let assetData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(frozen: 0, locked: 5_000_000),
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.unfreeze(resource: Resource.bandwidth.map())),
            assetData: assetData,
        )

        guard case let .stake(stake) = model.provider,
              case let .resource(resourceSelection) = stake.selection else { return }

        resourceSelection.selected = .energy
        model.onChangeResource(.bandwidth, .energy)
        model.amountInputModel.text = "2.0"
        model.onChangeAmountText("", "2.0")
        #expect(model.amountInputModel.isValid == true)

        resourceSelection.selected = .bandwidth
        model.onChangeResource(.energy, .bandwidth)
        model.amountInputModel.text = "2.0"
        model.onChangeAmountText("", "2.0")
        #expect(model.amountInputModel.isValid == false)
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
            type: .stake(.stake(validators: [validator1.map(), validator2.map()], validator: nil)),
            assetData: assetData,
        )

        model.amountInputModel.text = "1.5"
        model.onChangeAmountText("", "1.5")
        model.onValidatorSelected(validator2)

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
    func onAppearSetsMaxForFixedValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: 1_000_000))
        let assetData = AssetData.mock(asset: .mockBNB())
        let model = AmountSceneViewModel.mock(
            type: .stake(.withdraw(delegation: delegation.map())),
            assetData: assetData,
        )

        #expect(model.isInputDisabled == true)

        model.onAppear()
        #expect(model.amountInputModel.text.isEmpty == false)
    }
}

extension AmountSceneViewModel {
    static func mock(
        type: AmountType = .transfer(recipient: GemPaymentRecipient(recipient: .mock())),
        assetData: AssetData = .mock(balance: .mock()),
    ) -> AmountSceneViewModel {
        let model = AmountSceneViewModel(
            input: AmountInput(type: type, asset: assetData.asset),
            wallet: .mock(),
            service: GemAmountServiceMock(builder: GemAmountService.mock()),
            onTransferAction: { _ in },
        )
        model.assetQuery.value = assetData
        model.onChangeAssetBalance(assetData, assetData)
        return model
    }
}
