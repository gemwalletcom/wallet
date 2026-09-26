// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
@testable import Perpetuals
@testable import PerpetualsTestKit
import Primitives
import PrimitivesComponents
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
    func fieldStates() {
        let model = AutocloseSceneViewModel(type: .mock())

        let state = model.viewState

        #expect(state.takeProfit.tpslType.toPrimitives().autocloseTitle == "Take profit")
        #expect(state.stopLoss.tpslType.toPrimitives().autocloseTitle == "Stop loss")
        #expect(state.takeProfit.estimateTitle.text == "Expected profit")
        #expect(state.stopLoss.estimateTitle.text == "Expected loss")
        #expect(state.takeProfit.estimate == nil)
        #expect(model.percentSuggestions(state).map(\.value) == [15, 25, 50])
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
    func anOpeningPositionNamesTheSymbolAndItsDirection() throws {
        let row = { (data: AutocloseOpenData) in
            let model = AutocloseSceneViewModel(type: .mock(data: data))
            return model.positionRow(model.viewState)
        }
        let sized = try #require(row(.mock(symbol: "ETH", direction: .short, leverage: 5, size: 250)))
        let empty = try #require(row(.mock(size: 0)))

        #expect(sized.title == "ETH")
        #expect(sized.subtitle?.text.text == "SHORT 5x")
        #expect(sized.subtitle?.tone == .negative)
        guard case let .value(size, _) = sized.trailing else {
            Issue.record("a sized order shows its size")
            return
        }
        #expect(size.text.text == "$250.00")
        #expect(empty.trailing == .none)
    }

    @Test
    func openKeepsTheEnteredTrigger() {
        let model = AutocloseSceneViewModel(type: .mock(data: .mock(symbol: "BTC", direction: .long, marketPrice: 100, leverage: 10, size: 1, assetDecimals: 8, takeProfit: "110")))

        #expect(model.takeProfitText == "110")
        #expect(model.viewState.takeProfit.estimate != nil)
        #expect(model.confirmButtonType(model.viewState) == .primary(.normal))
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
