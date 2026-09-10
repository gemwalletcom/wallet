// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct CachedAsyncImage<Content: View>: View {
    @State private var phase: AsyncImagePhase

    private let request: ImageRequest?
    private let content: (AsyncImagePhase) -> Content

    public init(url: URL?, size: CGFloat? = nil, scale: CGFloat = 1, @ViewBuilder content: @escaping (AsyncImagePhase) -> Content) {
        let request = url.map { ImageRequest(url: $0, maxPixelSize: size.map { $0 * scale }, scale: scale) }
        self.request = request
        self.content = content
        _phase = State(wrappedValue: request.flatMap(ImageLoader.shared.cached).map { .success(Image(uiImage: $0)) } ?? .empty)
    }

    public init<I: View, P: View>(
        url: URL?,
        size: CGFloat? = nil,
        scale: CGFloat = 1,
        @ViewBuilder content: @escaping (Image) -> I,
        @ViewBuilder placeholder: @escaping () -> P,
    ) where Content == _ConditionalContent<I, P> {
        self.init(url: url, size: size, scale: scale) { phase in
            if case let .success(image) = phase {
                content(image)
            } else {
                placeholder()
            }
        }
    }

    public var body: some View {
        ZStack {
            content(phase)
        }
        .task(id: request) { await load() }
    }

    private func load() async {
        guard let request else {
            phase = .empty
            return
        }
        if let image = ImageLoader.shared.cached(request) {
            phase = .success(Image(uiImage: image))
            return
        }
        phase = .empty
        do {
            phase = try .success(Image(uiImage: await ImageLoader.shared.image(for: request)))
        } catch {
            phase = .failure(error)
        }
    }
}
