// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Support
import Testing

struct SupportMessageBubbleViewModelTests {
    private func model(
        content: String = "hello",
        sender: SupportMessageSender = .agent(.mock()),
        status: SupportMessageStatus = .sent,
        images: [SupportMessageImage] = [],
        onRetry: @escaping (SupportMessage) -> Void = { _ in },
        onImage: @escaping (SupportMessageImage) -> Void = { _ in },
    ) -> SupportMessageBubbleViewModel {
        SupportMessageBubbleViewModel(
            message: .mock(id: "1", content: content, sender: sender, status: status, images: images),
            retryAction: onRetry,
            imageAction: onImage,
        )
    }

    @Test
    func theDisplayTextAndLinksComeFromCore() {
        let model = model(content: "Open [the docs](https://gemwallet.com/docs) now")

        #expect(model.displayText == "Open now")
        #expect(model.links.map(\.url) == ["https://gemwallet.com/docs"])
        #expect(model.links.map(\.title) == ["the docs"])
        #expect(model.hasLinks)
        #expect(model.hasContent)
    }

    @Test
    func plainTextHasNoLinks() {
        let model = model(content: "no links here")

        #expect(model.hasLinks == false)
        #expect(model.hasDisplayText)
        #expect(model.hasContent)
    }

    @Test
    func anEmptyMessageWithAnImageStillHasContentToShow() {
        let model = model(content: "", images: [SupportMessageImage(id: "img", url: "https://gemwallet.com/a.png", thumbnailUrl: nil, fileName: nil, fileSize: nil, width: nil, height: nil)])

        #expect(model.hasContent == false)
        #expect(model.hasImages)
        #expect(model.imageURL(for: model.images[0])?.absoluteString == "https://gemwallet.com/a.png")
    }

    @Test
    func theBubbleSideFollowsTheSender() {
        #expect(model(sender: .user).alignment == .trailing)
        #expect(model(sender: .agent(.mock())).alignment == .leading)
    }

    @Test
    func theStatusCarriesTheTimeOnlyWhenSent() {
        guard case let .sent(time) = model(status: .sent).status else {
            Issue.record("expected a sent status")
            return
        }
        #expect(time.isNotEmpty)

        #expect(model(status: .sending).isSending)
        #expect(model(status: .failed).isFailed)
    }

    @Test
    func retryingSendsTheSameMessageBack() {
        let recorder = MessageRecorder()
        let model = model(status: .failed, onRetry: { recorder.record($0.id) })

        model.retry()

        #expect(recorder.ids == ["1"])
    }

    @Test
    func tappingAnImageReportsIt() {
        let recorder = MessageRecorder()
        let image = SupportMessageImage(id: "img", url: "https://gemwallet.com/a.png", thumbnailUrl: nil, fileName: nil, fileSize: nil, width: nil, height: nil)
        let model = model(images: [image], onImage: { recorder.record($0.id) })

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
