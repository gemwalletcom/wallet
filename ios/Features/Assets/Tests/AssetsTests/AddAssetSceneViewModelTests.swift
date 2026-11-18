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
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x1", isImmediate: false))

        model.onSubmitAddress()
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x1", isImmediate: true))

        model.setInput("0x2")
        #expect(model.input.address == "0x2")
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x2", isImmediate: true))

        model.onChangeAddress()
        #expect(model.loadTrigger == AddAssetLoadTrigger(chain: .ethereum, address: "0x2", isImmediate: true))

        model.input.address = nil
        model.onChangeAddress()
        #expect(model.loadTrigger == nil)
        #expect(model.buttonState == .disabled)
    }

    @Test
    func pastingTheSameAddressKeepsTheFoundToken() async {
        let model = AddAssetSceneViewModel.mock()

        model.setInput("0x1")
        await model.load()
        #expect(model.buttonState == .normal)

        model.setInput("0x1")
        #expect(model.buttonState == .normal)

        model.onSubmitAddress()
        #expect(model.buttonState == .normal)
    }
}
