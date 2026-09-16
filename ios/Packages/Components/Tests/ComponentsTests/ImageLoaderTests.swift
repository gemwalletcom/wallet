// Copyright (c). Gem Wallet. All rights reserved.

@testable import Components
import Testing
import UIKit

struct ImageLoaderTests {
    private let url = URL(string: "https://example.com/icon.png")!

    @Test
    func decodeDownsamplesToTheRequestedPixelSize() {
        let image = ImageLoader.decode(png(side: 200), request: ImageRequest(url: url, maxPixelSize: 44, scale: 2))

        #expect(image?.cgImage?.width == 44)
        #expect(image?.cgImage?.height == 44)
        #expect(image?.scale == 2)
        #expect(image?.size == CGSize(width: 22, height: 22))
    }

    @Test
    func decodeKeepsSmallImagesAndFullSizeRequests() {
        #expect(ImageLoader.decode(png(side: 20), request: ImageRequest(url: url, maxPixelSize: 44, scale: 1))?.cgImage?.width == 20)
        #expect(ImageLoader.decode(png(side: 200), request: ImageRequest(url: url, maxPixelSize: nil, scale: 1))?.cgImage?.width == 200)
    }

    @Test
    func decodeAppliesTheStoredOrientation() {
        let sideways = UIImage(cgImage: UIImage(data: png(side: 200, height: 100))!.cgImage!, scale: 1, orientation: .right).jpegData(compressionQuality: 1)!

        let full = ImageLoader.decode(sideways, request: ImageRequest(url: url, maxPixelSize: nil, scale: 1))
        let small = ImageLoader.decode(sideways, request: ImageRequest(url: url, maxPixelSize: 50, scale: 1))

        #expect(full?.cgImage?.width == 100)
        #expect(full?.cgImage?.height == 200)
        #expect(small?.cgImage?.width == 25)
        #expect(small?.cgImage?.height == 50)
    }

    @Test
    func decodeRejectsNonImageData() {
        #expect(ImageLoader.decode(Data("not an image".utf8), request: ImageRequest(url: url, maxPixelSize: nil, scale: 1)) == nil)
    }

    @Test
    func theStoredResponseIsDecodedByTheLoadAndNotByTheLookup() async throws {
        let cache = URLCache(memoryCapacity: 1024 * 1024, diskCapacity: 0)
        let loader = ImageLoader(cache: cache)
        let request = ImageRequest(url: url, maxPixelSize: 44, scale: 2)
        let response = try #require(HTTPURLResponse(url: url, statusCode: 200, httpVersion: nil, headerFields: nil))
        cache.storeCachedResponse(CachedURLResponse(response: response, data: png(side: 200)), for: URLRequest(url: url))

        #expect(loader.cached(request) == nil, "the lookup reads decoded images only")

        let image = try await loader.image(for: request)
        #expect(image.cgImage?.width == 44)
        #expect(loader.cached(request) === image, "the load leaves the decoded image for the next lookup")
    }

    @Test
    func aLocalFileIsDecodedByTheLoadWithoutNetwork() async throws {
        let loader = ImageLoader(cache: URLCache(memoryCapacity: 0, diskCapacity: 0))
        let file = URL.temporaryDirectory.appending(path: "\(UUID().uuidString).png")
        let request = ImageRequest(url: file, maxPixelSize: 44, scale: 2)
        try png(side: 200).write(to: file)
        defer { try? FileManager.default.removeItem(at: file) }

        #expect(loader.cached(request) == nil)

        let image = try await loader.image(for: request)
        #expect(image.cgImage?.width == 44)
        #expect(loader.cached(request) === image)
    }

    @Test
    func concurrentColdRequestsDecodeOnce() async throws {
        let loader = ImageLoader(cache: URLCache(memoryCapacity: 0, diskCapacity: 0))
        let file = URL.temporaryDirectory.appending(path: "\(UUID().uuidString).png")
        let request = ImageRequest(url: file, maxPixelSize: 44, scale: 2)
        try png(side: 200).write(to: file)
        defer { try? FileManager.default.removeItem(at: file) }

        async let first = loader.image(for: request)
        async let second = loader.image(for: request)
        let images = try await [first, second]

        #expect(images[0] === images[1], "both consumers get the one decoded image")
        #expect(loader.cached(request) === images[0])
    }

    @Test
    func cacheKeyDistinguishesSizeAndScale() {
        let small = ImageRequest(url: url, maxPixelSize: 44, scale: 2)
        let large = ImageRequest(url: url, maxPixelSize: 88, scale: 2)
        let full = ImageRequest(url: url, maxPixelSize: nil, scale: 3)

        #expect(small.cacheKey != large.cacheKey)
        #expect(small.cacheKey != full.cacheKey)
        #expect(small.cacheKey == ImageRequest(url: url, maxPixelSize: 44, scale: 2).cacheKey)
    }

    @Test
    func oneLoadIsSharedAndOutlivesACanceledConsumer() async throws {
        let loads = InFlightLoads()
        let request = ImageRequest(url: url, maxPixelSize: 44, scale: 2)
        let expected = try #require(ImageLoader.decode(png(side: 200), request: request))
        let loadCount = LoadCount()
        let load: @Sendable () async throws -> UIImage = {
            loadCount.increment()
            try await Task.sleep(for: .milliseconds(50))
            return expected
        }

        let shared = await loads.task(for: request, load: load)
        _ = await loads.task(for: request, load: load)

        let consumer = Task { try await shared.value }
        consumer.cancel()

        #expect(try await shared.value === expected, "a dismissed consumer does not cancel the load another consumer waits on")
        #expect(loadCount.value == 1, "the second request joins the load in flight")
    }

    private func png(side: Int, height: Int? = nil) -> Data {
        let format = UIGraphicsImageRendererFormat()
        format.scale = 1
        let size = CGSize(width: side, height: height ?? side)
        return UIGraphicsImageRenderer(size: size, format: format).pngData { context in
            UIColor.red.setFill()
            context.fill(CGRect(origin: .zero, size: size))
        }
    }
}

private final class LoadCount: @unchecked Sendable {
    private(set) var value = 0

    func increment() {
        value += 1
    }
}
