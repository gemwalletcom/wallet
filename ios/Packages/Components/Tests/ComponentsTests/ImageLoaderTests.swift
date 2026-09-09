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
    func cacheKeyDistinguishesSizeAndScale() {
        let small = ImageRequest(url: url, maxPixelSize: 44, scale: 2)
        let large = ImageRequest(url: url, maxPixelSize: 88, scale: 2)
        let full = ImageRequest(url: url, maxPixelSize: nil, scale: 3)

        #expect(small.cacheKey != large.cacheKey)
        #expect(small.cacheKey != full.cacheKey)
        #expect(small.cacheKey == ImageRequest(url: url, maxPixelSize: 44, scale: 2).cacheKey)
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
