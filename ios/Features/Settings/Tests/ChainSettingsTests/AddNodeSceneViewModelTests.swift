// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
@testable import Primitives
import Testing
@testable import Settings

@MainActor
struct AddNodeSceneViewModelTests {
    @Test
    func anUntouchedFormHasNothingToShow() {
        let model = AddNodeSceneViewModel(chain: .ethereum, service: GemChainSettingsServiceMock())

        #expect(model.state.isNoData)
    }

    @Test
    func aNodeThatAnswersFillsTheRows() async {
        let service = GemChainSettingsServiceMock()
        service.checkResult = .success(.mock(url: "https://node"))
        let model = AddNodeSceneViewModel(chain: .ethereum, service: service)

        model.setInput("https://node")
        await model.load()

        guard case let .data(result) = model.state else {
            Issue.record("Expected .data, got \(model.state)")
            return
        }
        #expect(result.url == "https://node")
        #expect(result.fields.count == 4)
    }

    @Test
    func anEmptyFieldAsksForNothing() {
        let model = AddNodeSceneViewModel(chain: .ethereum, service: GemChainSettingsServiceMock())

        model.setInput("https://node")
        #expect(model.loadTrigger?.url == "https://node")

        model.setInput("")
        #expect(model.loadTrigger == nil, "clearing the field cancels the pending check")
    }

    @Test
    func aFailedCheckShowsTheError() async {
        let service = GemChainSettingsServiceMock()
        service.checkResult = .failure(.InvalidNetworkId)
        let model = AddNodeSceneViewModel(chain: .ethereum, service: service)

        model.setInput("https://node")
        await model.load()

        guard case .error = model.state else {
            Issue.record("Expected .error, got \(model.state)")
            return
        }
    }
}
