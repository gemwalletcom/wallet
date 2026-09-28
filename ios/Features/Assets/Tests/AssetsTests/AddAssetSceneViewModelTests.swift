// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
import Primitives
import Testing

@MainActor
struct AddAssetSceneViewModelTests {
    @Test
    func loadTrigger() {
        let model = AddAssetSceneViewModel.mock()

        model.input.address = "0x1"
        model.onChangeAddress()
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x1", isImmediate: false, attempt: 0))

        model.onSubmitAddress()
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x1", isImmediate: true, attempt: 1))

        model.setInput("0x2")
        #expect(model.input.address == "0x2")
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x2", isImmediate: true, attempt: 2))

        model.onChangeAddress()
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x2", isImmediate: true, attempt: 2))

        model.setInput("0x2")
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x2", isImmediate: true, attempt: 3), "pasting the same address again looks it up again")

        model.input.address = nil
        model.onChangeAddress()
        #expect(model.loadTrigger == nil)
        #expect(model.buttonState == .disabled)
    }
}
