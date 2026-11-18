// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
@testable import Primitives
@testable import Settings
import Testing

@MainActor
struct AddNodeSceneViewModelTests {
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

        #expect(model.urlInputModel.error == nil)
        guard case .error = model.state else {
            Issue.record("Expected .error, got \(model.state)")
            return
        }
    }

    @Test
    func anInvalidUrlShowsTheErrorUnderTheField() async {
        let service = GemChainSettingsServiceMock()
        service.checkResult = .failure(.InvalidUrl)
        let model = AddNodeSceneViewModel(chain: .ethereum, service: service)

        model.setInput("https://rpc..example.com")
        await model.load()

        #expect(model.urlInputModel.error != nil)
        guard case .noData = model.state else {
            Issue.record("Expected .noData, got \(model.state)")
            return
        }
    }

    @Test
    func doneChecksTheTypedUrlAtOnce() {
        let model = AddNodeSceneViewModel(chain: .ethereum, service: GemChainSettingsServiceMock())

        model.urlInputModel.text = "https://node"
        model.onChangeInput()
        #expect(model.loadTrigger == AddNodeLoadTrigger(url: "https://node", isImmediate: false))

        model.onSubmitInput()
        #expect(model.loadTrigger == AddNodeLoadTrigger(url: "https://node", isImmediate: true))
    }

    @Test
    func pastingTheSameUrlKeepsTheCheck() async {
        let service = GemChainSettingsServiceMock()
        service.checkResult = .success(.mock(url: "https://node", chainId: "1", latestBlockNumber: 21_000_000, isInSync: true, latency: Primitives.Latency.mock(latencyType: .fast, value: 12).toGem()))
        let model = AddNodeSceneViewModel(chain: .ethereum, service: service)

        model.setInput("https://node")
        await model.load()
        #expect(model.buttonState == .normal)

        model.setInput("https://node")
        #expect(model.buttonState == .normal)
    }
}
