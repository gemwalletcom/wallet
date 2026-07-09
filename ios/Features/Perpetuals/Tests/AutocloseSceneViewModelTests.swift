// Copyright (c). Gem Wallet. All rights reserved.

@testable import Perpetuals
@testable import PerpetualsTestKit
import Primitives
import Testing

@MainActor
struct AutocloseSceneViewModelTests {
    @Test
    func isEditing() {
        let model = AutocloseSceneViewModel(type: .mockOpen())
        model.input.takeProfit.text = ""
        model.input.stopLoss.text = ""

        #expect(model.isEditing(field: nil) == false)
        #expect(model.isEditing(field: .takeProfit) == true)
        #expect(model.isEditing(field: .stopLoss) == true)

        model.input.takeProfit.text = "100"
        #expect(model.isEditing(field: .takeProfit) == false)
        #expect(model.isEditing(field: .stopLoss) == true)
    }
}
