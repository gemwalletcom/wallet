// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
@testable import Primitives
@testable import Settings
import Testing

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
        service.checkResult = .success(.mock(url: "https://node", chainId: "1", latestBlockNumber: 21_000_000, isInSync: true, latency: Primitives.Latency.mock(latencyType: .fast, value: 12).toGem()))
        let model = AddNodeSceneViewModel(chain: .ethereum, service: service)

        model.setInput("https://node")
        await model.load()

        guard case let .data(result) = model.state else {
            Issue.record("Expected .data, got \(model.state)")
            return
        }
        #expect(result.count == 4)
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

    @Test
    func submittingOrPastingTheSameUrlChecksItAgain() {
        let model = AddNodeSceneViewModel(chain: .ethereum, service: GemChainSettingsServiceMock())

        model.urlInputModel.text = "https://node"
        model.onChangeInput()
        let typed = model.loadTrigger
        #expect(typed?.isImmediate == false)

        model.onSubmitInput()
        let submitted = model.loadTrigger
        #expect(submitted?.isImmediate == true)
        #expect(submitted != typed, "done replaces the pending debounce with one immediate check")

        model.setInput("https://node")
        #expect(model.loadTrigger != submitted, "an identical paste is a fresh check")
    }
}
