// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
@testable import Primitives
import Testing
@testable import Settings

@MainActor
struct AddNodeSceneViewModelTests {
    private func check(url: String) -> GemNodeCheck {
        GemNodeCheck(url: url, chainId: "1", latestBlockNumber: 21_000_000, isInSync: true, latency: Latency(latencyType: .fast, value: 12))
    }

    @Test
    func anUntouchedFormHasNothingToShow() {
        let model = AddNodeSceneViewModel(chain: .ethereum, service: GemChainSettingsServiceMock())

        #expect(model.state.isNoData)
    }

    @Test
    func aNodeThatAnswersFillsTheRows() async {
        let service = GemChainSettingsServiceMock()
        service.checkResult = .success(check(url: "https://node"))
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
