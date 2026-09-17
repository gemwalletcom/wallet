// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstoneServices
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
@testable import Support
import SupportTestKit
import Testing

@MainActor
struct SupportChatSceneViewModelTests {
    @Test
    func anEmptyChatSaysSo() {
        let model = SupportChatSceneViewModel.mock()

        #expect(model.isEmpty)
        #expect(model.days.isEmpty)
    }

    @Test
    func theDaysGroupTheStoredMessages() {
        let model = SupportChatSceneViewModel.mock()
        model.query.value = [.mock(id: "a"), .mock(id: "b")]

        #expect(model.isEmpty == false)
        #expect(model.days.count == 1)
    }

    @Test
    func loadingSyncsFromTheLastAgentMessage() async {
        let service = GemSupportServiceMock()
        let model = SupportChatSceneViewModel.mock(service: service)
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
        let model = SupportChatSceneViewModel.mock(service: service)
        model.query.value = [.mock(id: "a", sender: .user)]

        await model.load()

        #expect(service.syncedTimestamps == [0])
    }

    @Test
    func aFailedSyncLeavesNoAlert() async {
        let service = GemSupportServiceMock()
        service.syncError = AnyError("offline")
        let model = SupportChatSceneViewModel.mock(service: service)

        await model.load()

        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func sendingTextReachesTheService() async {
        let service = GemSupportServiceMock()
        let model = SupportChatSceneViewModel.mock(service: service)

        model.sendText("hello")
        await settle { !service.sentTexts.isEmpty }

        #expect(service.sentTexts == ["hello"])
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedSendShowsTheError() async {
        let service = GemSupportServiceMock()
        service.sendError = AnyError("message rejected")
        let model = SupportChatSceneViewModel.mock(service: service)

        model.sendText("hello")
        await settle { model.isPresentingAlertMessage != nil }

        #expect(model.isPresentingAlertMessage?.message == "message rejected")
    }

    @Test
    func retryingSendsTheMessageAgain() async {
        let service = GemSupportServiceMock()
        let model = SupportChatSceneViewModel.mock(service: service)

        model.retry(.mock(id: "failed", status: .failed))
        await settle { !service.retriedMessageIds.isEmpty }

        #expect(service.retriedMessageIds == ["failed"])
    }

    @Test
    func openingAnImagePreviewAsksForTheLocalFile() async {
        let service = GemSupportServiceMock()
        let model = SupportChatSceneViewModel.mock(service: service)
        let image = SupportMessageImage.mock(url: "https://gemwallet.com/a.png")

        model.openPreview(image)
        await settle { model.previewURL != nil }

        #expect(service.requestedImageUrls == ["https://gemwallet.com/a.png"])
        #expect(model.previewURL?.path == "/tmp/support.png")
    }

    @Test
    func anImageWithNoUsableUrlIsIgnored() async {
        let service = GemSupportServiceMock()
        let model = SupportChatSceneViewModel.mock(service: service)
        let image = SupportMessageImage.mock(url: "")

        model.openPreview(image)
        await settle { false }

        #expect(service.requestedImageUrls.isEmpty)
        #expect(model.previewURL == nil)
    }

    @Test
    func leavingTheSceneClearsTheTypingAgent() {
        let typing = ObservableSupportTyping()
        typing.update(SupportTyping(status: .on, agent: .mock(name: "Gemma")))
        let model = SupportChatSceneViewModel.mock(typing: typing)

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
