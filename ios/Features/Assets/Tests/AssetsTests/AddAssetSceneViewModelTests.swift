// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
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

        model.state = .loading
        model.input.address = nil
        model.onChangeAddress()
        #expect(model.loadTrigger == nil)
        #expect(model.state.isNoData)
    }
}

// MARK: - Mock Extensions

extension AddAssetSceneViewModel {
    static func mock() -> AddAssetSceneViewModel {
        AddAssetSceneViewModel(
            wallet: .mock(accounts: [.mock(chain: .ethereum)]),
            service: GemAddAssetServiceMock(),
        )
    }
}
