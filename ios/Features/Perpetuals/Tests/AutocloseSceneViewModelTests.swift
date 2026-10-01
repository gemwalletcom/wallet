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
        model.takeProfitText = ""
        model.stopLossText = ""

        #expect(model.isEditing(field: nil) == false)
        #expect(model.isEditing(field: .takeProfit) == true)
        #expect(model.isEditing(field: .stopLoss) == true)

        model.takeProfitText = "100"
        #expect(model.isEditing(field: .takeProfit) == false)
        #expect(model.isEditing(field: .stopLoss) == true)
    }

    @Test
    func percentFillsTheFocusedField() {
        let model = AutocloseSceneViewModel(type: .mock())
        model.onChangeFocusField(nil, .takeProfit)

        model.onSelectPercent(50)

        #expect(model.takeProfitText.isNotEmpty)
        #expect(model.viewState.takeProfit.estimate != nil)
        #expect(model.stopLossText.isEmpty)
    }

    @Test
    func aTransferCoreRefusesShowsTheError() {
        var transfers = 0
        let data = PerpetualPositionData.mock(
            perpetual: .mock(identifier: "BTC", price: 1000),
            position: .mock(size: 1, sizeValue: 1000, leverage: 10, entryPrice: 1000, marginType: .isolated, direction: .long, marginAmount: 100),
        )
        let model = AutocloseSceneViewModel(type: .modify(data, onTransferAction: { _ in transfers += 1 }))
        model.takeProfitText = "1500"

        model.onSelectConfirm()

        #expect(transfers == 0)
        #expect(model.isPresentingAlertMessage != nil)
    }
}
