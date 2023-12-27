package dev.birb.wgpu.mixin;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.model.TexturedModelData;
import net.minecraft.client.render.entity.model.EntityModelLayer;
import net.minecraft.client.render.entity.model.EntityModels;
import net.minecraft.client.texture.TextureManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

import static net.minecraft.screen.PlayerScreenHandler.BLOCK_ATLAS_TEXTURE;

@Mixin(TitleScreen.class)
public class TitleScreenMixin {
    @Unique
    private static final Gson GSON = new Gson();

    @Unique
    private boolean updatedTitle = false;

    @Inject(method = "render", at = @At("HEAD"))
    private void render(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        if (!updatedTitle && Wgpu.isInitialized()) {
            WgpuNative.cacheBlockStates();
            MinecraftClient.getInstance().updateWindowTitle();
            updatedTitle = true;

            long millis = System.currentTimeMillis();

            Map<EntityModelLayer, TexturedModelData> models = EntityModels.getModels();

            JsonObject json = new JsonObject();
            models.forEach((layer, data) -> json.add(layer.toString(), GSON.toJsonTree(data)));
            WgpuNative.registerEntities(json.toString());

            WgpuMcMod.MAY_INJECT_PART_IDS = true;

            WgpuMcMod.LOGGER.info("Uploaded " + models.size() + " TMDs to wgpu-mc and processed them in " + (System.currentTimeMillis() - millis) + "ms");

            TextureManager textureManager = MinecraftClient.getInstance().getTextureManager();
            int blockTexAtlasId = textureManager.getTexture(BLOCK_ATLAS_TEXTURE).getGlId();

            WgpuNative.identifyGlTexture(0, blockTexAtlasId);

            WgpuMcMod.ENTITIES_UPLOADED = true;
        }
    }
}
