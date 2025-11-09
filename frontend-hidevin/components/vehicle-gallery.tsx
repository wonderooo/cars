"use client"

import {useState} from "react"
import Image from "next/image"
import {Card} from "@/components/ui/card"
import {Button} from "@/components/ui/button"
import {ChevronLeft, ChevronRight, Maximize2} from "lucide-react"
import {Dialog, DialogContent, DialogTitle} from "@/components/ui/dialog"
import {VisuallyHidden} from "@radix-ui/react-visually-hidden"
import {VehicleImage} from "@/app/vin/[vin]/page";


interface VehicleGalleryProps {
    images: VehicleImage[]
}

function fullImageUrl(key: string | null): string {
    return key ? `https://cdn.carauctions24.eu/${key}` : "/placeholder.svg"
}

export function VehicleGallery({images}: VehicleGalleryProps) {
    const [selectedIndex, setSelectedIndex] = useState(0)
    const [isFullscreen, setIsFullscreen] = useState(false)

    const handlePrevious = () => {
        setSelectedIndex((prev) => (prev === 0 ? images.length - 1 : prev - 1))
    }

    const handleNext = () => {
        setSelectedIndex((prev) => (prev === images.length - 1 ? 0 : prev + 1))
    }

    if (!images || images.length === 0) {
        return (
            <Card className="p-8 text-center">
                <p className="text-muted-foreground">No images available</p>
            </Card>
        )
    }

    return (
        <div className="space-y-4">
            {/* Main Image */}
            <Card className="relative group">
                <div className="aspect-video relative bg-muted overflow-hidden rounded-lg">
                    {
                        images[selectedIndex].highResMimeType?.startsWith("video") ? (
                            <video
                                src={fullImageUrl(images[selectedIndex].highResBucketKey)}
                                autoPlay
                                loop
                                playsInline
                                className="w-full h-full"
                            />
                        ) : (
                            <Image
                                src={fullImageUrl(images[selectedIndex].highResBucketKey)}
                                alt={`Vehicle image ${selectedIndex + 1}`}
                                fill
                                className="object-cover"
                                priority
                            />
                        )
                    }

                    {/* Navigation Arrows */}
                    {images.length > 1 && (
                        <>
                            <Button
                                variant="secondary"
                                size="icon"
                                className="absolute left-4 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity"
                                onClick={handlePrevious}
                            >
                                <ChevronLeft className="h-4 w-4"/>
                            </Button>
                            <Button
                                variant="secondary"
                                size="icon"
                                className="absolute right-4 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity"
                                onClick={handleNext}
                            >
                                <ChevronRight className="h-4 w-4"/>
                            </Button>
                        </>
                    )}

                    {/* Fullscreen Button */}
                    <Button
                        variant="secondary"
                        size="icon"
                        className="absolute top-4 right-4 opacity-0 group-hover:opacity-100 transition-opacity"
                        onClick={() => setIsFullscreen(true)}
                    >
                        <Maximize2 className="h-4 w-4"/>
                    </Button>

                    {/* Image Counter */}
                    <div
                        className="absolute bottom-4 right-4 bg-background/80 backdrop-blur-sm px-3 py-1 rounded-full text-sm">
                        {selectedIndex + 1} / {images.length}
                    </div>
                </div>
            </Card>

            {/* Thumbnail Grid */}
            {images.length > 1 && (
                <div className="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 gap-2">
                    {images.map((image, index) => (
                        <button
                            key={image.sequenceNumber}
                            onClick={() => setSelectedIndex(index)}
                            className={`relative aspect-video rounded-lg overflow-hidden border-2 transition-all ${
                                index === selectedIndex
                                    ? "border-primary ring-2 ring-primary/20"
                                    : "border-transparent hover:border-muted-foreground/50"
                            }`}
                        >
                            <Image
                                src={fullImageUrl(image.thumbnailBucketKey)}
                                alt={`Thumbnail ${index + 1}`}
                                fill
                                className="object-cover"
                            />
                        </button>
                    ))}
                </div>
            )}

            {/* Fullscreen Dialog */}
            <Dialog open={isFullscreen} onOpenChange={setIsFullscreen}>
                <DialogContent className="max-w-[95vw] w-full h-[95vh] p-2 flex items-center justify-center">
                    <VisuallyHidden>
                        <DialogTitle>Vehicle Image Gallery</DialogTitle>
                    </VisuallyHidden>

                    <div className="relative w-full h-full flex items-center justify-center">
                        <div className="relative w-full h-full">
                            <Image
                                src={fullImageUrl(images[selectedIndex].highResBucketKey)}
                                alt={`Vehicle image ${selectedIndex + 1}`}
                                fill
                                className="object-contain"
                            />
                        </div>

                        {images.length > 1 && (
                            <>
                                <Button
                                    variant="secondary"
                                    size="icon"
                                    className="absolute left-4 top-1/2 -translate-y-1/2 z-10"
                                    onClick={handlePrevious}
                                >
                                    <ChevronLeft className="h-4 w-4"/>
                                </Button>
                                <Button
                                    variant="secondary"
                                    size="icon"
                                    className="absolute right-4 top-1/2 -translate-y-1/2 z-10"
                                    onClick={handleNext}
                                >
                                    <ChevronRight className="h-4 w-4"/>
                                </Button>
                            </>
                        )}
                    </div>
                </DialogContent>
            </Dialog>
        </div>
    )
}
