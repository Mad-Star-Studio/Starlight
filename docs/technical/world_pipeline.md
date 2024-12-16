# World Pipeline documentation

This document describes the world pipeline, which is the process of generating and presenting the world in the game.

## Overview

The world pipeline has these steps:

1. ***Scripting Engine***: The scripting engine is responsible for managing and running mod/game code.
    > Mutates: Game state, World state
    > May *immediately* cause generation/loading of chunks.
    - Submits `ObservationInShouldLoad` events to the event queue if a chunk was loaded/generated by this.
    - Submits `WorldChunkUpdate` events to the event queue if a chunk was updated by this, but also already existed.
2. ***Observation***: We start by determining what the player can see, and from this determine what is to be loaded and what is to be unloaded.
    > No mutations
    - Consumes `ObservationInShouldLoad`
    - Submits `ObservationLoad` and `ObservationUnload`
3. ***Management***: We orchestrate the storage of chunks in the world, and keep track of what is loaded and what is not.
    > This is responsible for maintaining a buffer of loaded chunks around those presented.
    - Consumes `ObservationLoad` and `ObservationUnload`
    - Submits `WorldMgrLoad` and `WorldMgrUnload`
    - Submits `WorldMgrChunkReady` when a chunk is ready to be presented.
4. ***Population***: We load or generate contents of chunks into memory.
    > Mutates: World chunk list
    - Consumes `WorldMgrLoad`
    - If we successfully generate a chunk, Submits a `WorldGenGenerateSuccess` event to the event queue.
    - If we successfully load a chunk from storage, Submits a `WorldGenRestoreSuccess` event to the event queue.
    - Either way, we also submit a `WorldGenLoadSuccess` event to the event queue.
5. ***Simulation***: Simulate the world, mutating the state of chunks.
    > Mutates: World chunk content state
    > We iterate over all loaded chunks and simulate them.
    - Submits one or more `WorldChunkUpdate` events to the event queue if a chunk is updated.
6. ***Cleanup***: We unload chunks that are no longer needed gracefully.
    > Mutates: Persistent storage
    - Consumes `WorldMgrUnload`
    - Submits `WorldCleanUnloadSuccess` events to the event queue.
7. ***Physics***: We simulate the physics of entities in the world.
    > Mutates: Entity state
    - Consumes no events.
    - Submits `WorldPhysCollisionWithEntity` events to the event queue if entities collide with each other.
    - Submits `WorldPhysCollisionWithWorld` events to the event queue if entities collide with the world.
8. ***(CLIENT) Presentation***: We manage what chunks needs to be drawn.
    > No mutations
    - Consumes `WorldGenLoadSuccess` and `WorldCleanUnloadSuccess`
    - Consumes `WorldChunkUpdate`
    - Submits `WorldPresentationAdd` and `WorldPresentationRemove` events to the event queue.
    - Submits `WorldPresentationUpdate` events to the event queue if an existing chunk is updated (mutually exclusive with `WorldPresentationAdd`).
9. ***(CLIENT) Meshing***: We generate meshes for chunks that need to be drawn and commit them to Bevy Engine.
    > Mutates: Bevy Engine ECS
    - Consumes `WorldPresentationAdd`
        > Generates a mesh for the chunk and adds it to the ECS Waypoint 1as an entity.
    - Consumes `WorldPresentationRemove`
        > Removes the mesh entity from the ECS.
    - Consumes `WorldPresentationUpdate`
        > Fetches the chunk from the ECS and updates the mesh.