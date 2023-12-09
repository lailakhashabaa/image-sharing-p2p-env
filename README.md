# Cloud P2P Environment for Controlled Sharing of Images

## Introduction

This project focuses on applying design concepts, with a primary emphasis on transparency in load balancing and fault tolerance. It involves implementing a distributed election algorithm to support cloud qualities. The cloud will offer two main functionalities: encryption of user images through steganography and a user-oriented discovery service.

## Functionality

### Users

- Users own images and can register with the cloud's discovery service.
- Users can choose to encrypt their images through steganography before sharing.
- Encryption requests are generated, and images are sent to the cloud for encryption.
- Clients can inquire about their peers through the discovery service.
- Users can request lower resolution images from peers with specified viewing quotas.

### Image Management

- Users can only see their own images or images with hidden usernames.
- Image owners can update image data, add/remove users, or change viewing quotas.
- The number of views is updated inside the image, and access is denied when views are consumed.
- Offline operation is supported with a best-effort policy to update views.

### Cloud Services

- The cloud provides encryption and a discovery service using a P2P architecture.
- Load balancing is implemented by having the client's middleware multicast requests to three cloud servers.
- Servers use a distributed election algorithm considering factors like current load and CPU usage
- Periodic server failures are simulated by having one server ignore communication, creating a failure effect for up to 20 seconds.
