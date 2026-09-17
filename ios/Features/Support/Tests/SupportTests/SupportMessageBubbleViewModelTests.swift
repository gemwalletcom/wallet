// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Support
@testable import SupportTestKit
import Testing

struct SupportMessageBubbleViewModelTests {
    @Test
    func theDisplayTextAndLinksComeFromCore() {
        let model = SupportMessageBubbleViewModel.mock(message: .mock(content: "Open [the docs](https://gemwallet.com/docs) now"))

        #expect(model.displayText == "Open now")
        #expect(model.links.map(\.url) == ["https://gemwallet.com/docs"])
        #expect(model.links.map(\.title) == ["the docs"])
        #expect(model.hasLinks)
        #expect(model.hasContent)
    }

    @Test
    func plainTextHasNoLinks() {
        let model = SupportMessageBubbleViewModel.mock(message: .mock(content: "no links here"))

        #expect(model.hasLinks == false)
        #expect(model.hasDisplayText)
        #expect(model.hasContent)
    }

    @Test
    func anEmptyMessageWithAnImageStillHasContentToShow() {
        let model = SupportMessageBubbleViewModel.mock(message: .mock(content: "", images: [.mock()]))

        #expect(model.hasContent == false)
        #expect(model.hasImages)
        #expect(model.imageURL(for: model.images[0])?.absoluteString == "https://gemwallet.com/a.png")
    }

    @Test
    func theBubbleSideFollowsTheSender() {
        #expect(SupportMessageBubbleViewModel.mock(message: .mock(sender: .user)).alignment == .trailing)
        #expect(SupportMessageBubbleViewModel.mock(message: .mock(sender: .agent(.mock()))).alignment == .leading)
    }

    @Test
    func theStatusCarriesTheTimeOnlyWhenSent() {
        guard case let .sent(time) = SupportMessageBubbleViewModel.mock(message: .mock(status: .sent)).status else {
            Issue.record("expected a sent status")
            return
        }
        #expect(time.isNotEmpty)

        #expect(SupportMessageBubbleViewModel.mock(message: .mock(status: .sending)).isSending)
        #expect(SupportMessageBubbleViewModel.mock(message: .mock(status: .failed)).isFailed)
    }

    @Test
    func retryingSendsTheSameMessageBack() {
        let recorder = MessageRecorder()
        let model = SupportMessageBubbleViewModel.mock(message: .mock(id: "1", status: .failed), retryAction: { recorder.record($0.id) })

        model.retry()

        #expect(recorder.ids == ["1"])
    }

    @Test
    func tappingAnImageReportsIt() {
        let recorder = MessageRecorder()
        let image = SupportMessageImage.mock(id: "img")
        let model = SupportMessageBubbleViewModel.mock(message: .mock(images: [image]), imageAction: { recorder.record($0.id) })

        model.onImageTap(image)

        #expect(recorder.ids == ["img"])
    }
}

private final class MessageRecorder: @unchecked Sendable {
    private(set) var ids: [String] = []

    func record(_ id: String) {
        ids.append(id)
    }
}
