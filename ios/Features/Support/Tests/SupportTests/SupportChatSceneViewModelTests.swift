// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
@testable import Support
import Testing

@MainActor
struct SupportChatSceneViewModelTests {
    private func model(service: GemSupportServiceMock = GemSupportServiceMock(), typing: ObservableSupportTyping = ObservableSupportTyping()) -> SupportChatSceneViewModel {
        SupportChatSceneViewModel(service: service, typing: typing)
    }

    @Test
    func anEmptyChatSaysSo() {
        let model = model()

        #expect(model.isEmpty)
        #expect(model.days.isEmpty)
    }

    @Test
    func theDaysGroupTheStoredMessages() {
        let model = model()
        model.query.value = [.mock(id: "a"), .mock(id: "b")]

        #expect(model.isEmpty == false)
        #expect(model.days.count == 1)
    }

    @Test
    func loadingSyncsFromTheLastAgentMessage() async {
        let service = GemSupportServiceMock()
        let model = model(service: service)
        model.query.value = [
            .mock(id: "a", sender: .user, createdAt: Date(timeIntervalSince1970: 100)),
            .mock(id: "b", sender: .agent(.mock()), createdAt: Date(timeIntervalSince1970: 200)),
            .mock(id: "c", sender: .user, createdAt: Date(timeIntervalSince1970: 300)),
        ]

        await model.load()

        #expect(service.syncedTimestamps == [200])
    }

    @Test
    func aChatWithNoAgentReplySyncsFromTheStart() async {
        let service = GemSupportServiceMock()
        let model = model(service: service)
        model.query.value = [.mock(id: "a", sender: .user)]

        await model.load()

        #expect(service.syncedTimestamps == [0])
    }

    @Test
    func aFailedSyncLeavesNoAlert() async {
        let service = GemSupportServiceMock()
        service.syncError = AnyError("offline")
        let model = model(service: service)

        await model.load()

        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func sendingTextReachesTheService() async {
        let service = GemSupportServiceMock()
        let model = model(service: service)

        model.sendText("hello")
        await settle { !service.sentTexts.isEmpty }

        #expect(service.sentTexts == ["hello"])
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedSendShowsTheError() async {
        let service = GemSupportServiceMock()
        service.sendError = AnyError("message rejected")
        let model = model(service: service)

        model.sendText("hello")
        await settle { model.isPresentingAlertMessage != nil }

        #expect(model.isPresentingAlertMessage?.message == "message rejected")
    }

    @Test
    func retryingSendsTheMessageAgain() async {
        let service = GemSupportServiceMock()
        let model = model(service: service)

        model.retry(.mock(id: "failed", status: .failed))
        await settle { !service.retriedMessageIds.isEmpty }

        #expect(service.retriedMessageIds == ["failed"])
    }

    @Test
    func openingAnImagePreviewAsksForTheLocalFile() async {
        let service = GemSupportServiceMock()
        let model = model(service: service)
        let image = SupportMessageImage(id: "img", url: "https://gemwallet.com/a.png", thumbnailUrl: nil, fileName: nil, fileSize: nil, width: nil, height: nil)

        model.openPreview(image)
        await settle { model.previewURL != nil }

        #expect(service.requestedImageUrls == ["https://gemwallet.com/a.png"])
        #expect(model.previewURL?.path == "/tmp/support.png")
    }

    @Test
    func anImageWithNoUsableUrlIsIgnored() async {
        let service = GemSupportServiceMock()
        let model = model(service: service)
        let image = SupportMessageImage(id: "img", url: "", thumbnailUrl: nil, fileName: nil, fileSize: nil, width: nil, height: nil)

        model.openPreview(image)
        await settle { false }

        #expect(service.requestedImageUrls.isEmpty)
        #expect(model.previewURL == nil)
    }

    @Test
    func leavingTheSceneClearsTheTypingAgent() {
        let typing = ObservableSupportTyping()
        typing.update(SupportTyping(status: .on, agent: .mock(name: "Gemma")))
        let model = model(typing: typing)

        #expect(model.typingAgentName == "Gemma")

        model.onDisappear()

        #expect(model.typingAgentName == nil)
    }

    private func settle(until condition: () -> Bool) async {
        for _ in 0 ..< 60 {
            await Task.yield()
            if condition() { return }
            try? await Task.sleep(for: .milliseconds(5))
        }
    }
}
