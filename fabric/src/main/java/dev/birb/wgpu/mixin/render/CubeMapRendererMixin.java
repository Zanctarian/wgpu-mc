package dev.birb.wgpu.mixin.render;

import java.nio.FloatBuffer;

import javax.annotation.Nullable;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import com.mojang.blaze3d.systems.RenderSystem;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.CubeMapRenderer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormats;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.Matrix4f;
import net.minecraft.util.math.Vec3f;

@Mixin(CubeMapRenderer.class)
public class CubeMapRendererMixin {
    
    @Shadow @Final private Identifier[] faces;

    /**
     * @author wgpu-mc
     */
    public void draw(MinecraftClient client, float x, float y, float alpha) {
        Tessellator tessellator = Tessellator.getInstance();
        BufferBuilder bufferBuilder = tessellator.getBuffer();

        Matrix4f mat = Matrix4f.viewboxMatrix(85.0D, client.getWindow().getFramebufferWidth() / client.getWindow().getFramebufferHeight(), 0.05F, 10.0F);
        RenderSystem.backupProjectionMatrix();
        RenderSystem.setProjectionMatrix(mat);

        MatrixStack matrixStack = RenderSystem.getModelViewStack();
        matrixStack.push();
        matrixStack.loadIdentity();
        //flip images
        matrixStack.multiply(Vec3f.POSITIVE_X.getDegreesQuaternion(180.0F));
        RenderSystem.applyModelViewMatrix();

        RenderSystem.setShader(GameRenderer::getPositionTexColorShader);
        //Commented out to see easier. Motion blur effect used in vanilla.
        //for(int i = 0; i < 4; i++) {
          matrixStack.push();
          
          //float rx = ((i % 2) / 2.0F - 0.5F) / 256.0F;
          //float ry = ((i / 2) / 2.0F - 0.5F) / 256.0F;
          
          //motion blur -- might need to tone down? Depends if you get it to work right.
          //matrixStack.translate(rx, ry, 0.0F);

          //matrixStack.multiply(Vec3f.POSITIVE_X.getDegreesQuaternion(x));
          matrixStack.multiply(Vec3f.POSITIVE_Y.getDegreesQuaternion(y));
          RenderSystem.applyModelViewMatrix();

          for(int side = 0; side < 6; side++) {
              RenderSystem.setShaderTexture(0, this.faces[side]);
              bufferBuilder.begin(VertexFormat.DrawMode.QUADS, VertexFormats.POSITION_TEXTURE_COLOR);
              int l = Math.round(255.0F * alpha); // / (i + 1);
              if (side == 0) {
                bufferBuilder.vertex(-1.0D, -1.0D, 1.0D).texture(0.0F, 0.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, 1.0D, 1.0D).texture(0.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, 1.0D, 1.0D).texture(1.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, -1.0D, 1.0D).texture(1.0F, 0.0F).color(255, 255, 255, l).next();
              } 
              if (side == 1) {
                bufferBuilder.vertex(1.0D, -1.0D, 1.0D).texture(0.0F, 0.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, 1.0D, 1.0D).texture(0.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, 1.0D, -1.0D).texture(1.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, -1.0D, -1.0D).texture(1.0F, 0.0F).color(255, 255, 255, l).next();
              } 
              if (side == 2) {
                bufferBuilder.vertex(1.0D, -1.0D, -1.0D).texture(0.0F, 0.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, 1.0D, -1.0D).texture(0.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, 1.0D, -1.0D).texture(1.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, -1.0D, -1.0D).texture(1.0F, 0.0F).color(255, 255, 255, l).next();
              } 
              if (side == 3) {
                bufferBuilder.vertex(-1.0D, -1.0D, -1.0D).texture(0.0F, 0.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, 1.0D, -1.0D).texture(0.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, 1.0D, 1.0D).texture(1.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, -1.0D, 1.0D).texture(1.0F, 0.0F).color(255, 255, 255, l).next();
              } 
              if (side == 4) {
                bufferBuilder.vertex(-1.0D, -1.0D, -1.0D).texture(0.0F, 0.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, -1.0D, 1.0D).texture(0.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, -1.0D, 1.0D).texture(1.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, -1.0D, -1.0D).texture(1.0F, 0.0F).color(255, 255, 255, l).next();
              } 
              if (side == 5) {
                bufferBuilder.vertex(-1.0D, 1.0D, 1.0D).texture(0.0F, 0.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(-1.0D, 1.0D, -1.0D).texture(0.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, 1.0D, -1.0D).texture(1.0F, 1.0F).color(255, 255, 255, l).next();
                bufferBuilder.vertex(1.0D, 1.0D, 1.0D).texture(1.0F, 0.0F).color(255, 255, 255, l).next();
              } 
              tessellator.draw();
          }
          matrixStack.pop();
          RenderSystem.applyModelViewMatrix();
       
        //}
        RenderSystem.restoreProjectionMatrix();
        matrixStack.pop();

        RenderSystem.applyModelViewMatrix();
    }
}
