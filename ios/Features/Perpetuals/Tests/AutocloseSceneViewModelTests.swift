// Copyright (c). Gem Wallet. All rights reserved.

@testable import Perpetuals
@testable import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct AutocloseSceneViewModelTests {
    @Test
    func isEditing() {
        let model = AutocloseSceneViewModel(type: .mock())
        model.input.takeProfit.text = ""
        model.input.stopLoss.text = ""

        #expect(model.isEditing(field: nil) == false)
        #expect(model.isEditing(field: .takeProfit) == true)
        #expect(model.isEditing(field: .stopLoss) == true)

        model.input.takeProfit.text = "100"
        #expect(model.isEditing(field: .takeProfit) == false)
        #expect(model.isEditing(field: .stopLoss) == true)
    }

    @Test
    func fieldStates() {
        let model = AutocloseSceneViewModel(type: .mock())

        #expect(model.takeProfitModel.title == "Take profit")
        #expect(model.stopLossModel.title == "Stop loss")
        #expect(model.takeProfitModel.profitTitle == "Expected profit")
        #expect(model.stopLossModel.profitTitle == "Expected loss")
        #expect(model.takeProfitModel.expectedPnL == "-")
        #expect(model.takeProfitModel.percentSuggestions.map(\.value) == [15, 25, 50])
    }

    @Test
    func percentFillsTheFocusedField() {
        let model = AutocloseSceneViewModel(type: .mock())
        model.onChangeFocusField(nil, .takeProfit)

        model.onSelectPercent(50)

        #expect(model.input.takeProfit.text.isNotEmpty)
        #expect(model.takeProfitModel.expectedPnL != "-")
        #expect(model.input.stopLoss.text.isEmpty)
    }

    @Test
    func openKeepsTheEnteredTrigger() {
        let model = AutocloseSceneViewModel(type: .mock(data: .mock(takeProfit: "110")))

        #expect(model.input.takeProfit.text == "110")
        #expect(model.takeProfitModel.expectedPnL != "-")
        #expect(model.confirmButtonType == .primary(.normal))
    }

    @Test
    func aTransferCoreRefusesShowsTheError() {
        var transfers = 0
        let data = PerpetualPositionData.mock(perpetual: .mock(identifier: "BTC"))
        let model = AutocloseSceneViewModel(type: .modify(data, onTransferAction: { _ in transfers += 1 }))
        model.input.takeProfit.text = "1500"
        model.onChangePrice()

        model.onSelectConfirm()

        #expect(transfers == 0)
        #expect(model.isPresentingAlertMessage != nil)
    }
}
